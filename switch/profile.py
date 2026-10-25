#!/usr/bin/env python3
"""Names the profiler's samples (crates/benilla-horizon/sampler.c, WOWNX_PROFILE=1): per thread, the functions its
PCs were in and the callers its LRs were in, most samples first, from the ELF's symbols; and for a thread waiting on a
held lock, which lock (a data symbol, or a heap address) and the return addresses above libnx's mutexLock.

Usage: switch/profile.py samples.txt [benilla.elf] [--top N]
"""
import bisect
import collections
import subprocess
import sys

NM = "/opt/devkitpro/devkitA64/bin/aarch64-none-elf-nm"


def symbols(elf, kinds):
    out = subprocess.run([NM, "-n", "-C", "--defined-only", elf], capture_output=True, text=True, check=True).stdout
    addrs, names = [], []
    for line in out.splitlines():
        parts = line.split(" ", 2)
        if len(parts) == 3 and parts[1] in kinds:
            addrs.append(int(parts[0], 16))
            names.append(parts[2])
    return addrs, names


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    top = int(sys.argv[sys.argv.index("--top") + 1]) if "--top" in sys.argv else 25
    if "--top" in sys.argv:
        args.remove(str(top))
    samples = args[0]
    elf = args[1] if len(args) > 1 else "target/aarch64-nintendo-switch-newlib/switch/benilla.elf"
    addrs, names = symbols(elf, "tTwW")
    data_addrs, data_names = symbols(elf, "dDbBrRsS")
    # The program's end: a lock past it is on the heap (the sampler keeps a heap address as it is).
    end = max(data_addrs + addrs) + (1 << 20)

    def name(offset):
        i = bisect.bisect_right(addrs, offset) - 1
        return names[i] if i >= 0 else f"?{offset:x}"

    def lock_name(offset):
        if offset > end:
            return f"heap 0x{offset:x}"
        i = bisect.bisect_right(data_addrs, offset) - 1
        return f"{data_names[i]}+0x{offset - data_addrs[i]:x}" if i >= 0 else f"?{offset:x}"

    totals = {}
    counts = collections.defaultdict(collections.Counter)
    header = ""
    for line in open(samples):
        if line.startswith("#"):
            header = line.strip()
            continue
        parts = line.split()
        if parts[0] == "thread":
            totals[int(parts[1])] = int(parts[2])
            continue
        thread, kind, offset, n = int(parts[0]), parts[1], int(parts[2], 16), int(parts[3])
        counts[(thread, kind)][lock_name(offset) if kind == "M" else name(offset)] += n

    print(header)
    for thread in sorted(totals, key=lambda t: -totals[t]):
        total = totals[thread]
        if not total:
            continue
        pcs = counts[(thread, "P")]
        # A thread that mostly waits says so in its first line; the busy ones are worth reading.
        print(f"\n== thread {thread}: {total} samples")
        for fn, n in pcs.most_common(top):
            print(f"  {100 * n / total:5.1f}%  {fn[:150]}")
        print("  -- callers (LR)")
        for fn, n in counts[(thread, "L")].most_common(top // 2):
            print(f"  {100 * n / total:5.1f}%  {fn[:150]}")
        if counts[(thread, "M")]:
            print("  -- waiting on a held lock: the lock")
            for lock, n in counts[(thread, "M")].most_common(8):
                print(f"  {100 * n / total:5.1f}%  {lock[:150]}")
            for depth in "123":
                print(f"  -- ... return address {depth} above mutexLock")
                for fn, n in counts[(thread, depth)].most_common(8):
                    print(f"  {100 * n / total:5.1f}%  {fn[:150]}")


if __name__ == "__main__":
    main()
