# copy_data_ftp.py ("data to FTP"): a developer's copy of a WoW client's Data folder to the console's FTP server, for
# tests on the Switch (the NRO reads sdmc:/wow-nx/Data). Players copy the folder to the SD card themselves.
#
#   python3 switch/copy_data_ftp.py <Data folder> <ip> [--dest /wow-nx/Data] [--port 5000] [--skip PATTERN]...
#
# Resumable: a remote file that already has the right size is skipped, so after an interruption run it again.
# Anonymous login, as the console's FTP servers use. --skip skips paths matching a glob (case-insensitive).
import argparse, fnmatch, ftplib, os, sys, time


def connect(ip, port):
    f = ftplib.FTP()
    f.connect(ip, port, timeout=120)
    f.login("anonymous", "anonymous")
    f.voidcmd("TYPE I")
    return f


def make_dirs(f, folder, made):
    parts = folder.strip("/").split("/")
    for i in range(1, len(parts) + 1):
        path = "/" + "/".join(parts[:i])
        if path not in made:
            try:
                f.mkd(path)
            except ftplib.error_perm:
                pass  # already there
            made.add(path)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("src")
    ap.add_argument("ip")
    ap.add_argument("--dest", default="/wow-nx/Data")
    ap.add_argument("--port", type=int, default=5000)
    ap.add_argument("--skip", action="append", default=[])
    a = ap.parse_args()
    dst = "/" + a.dest.strip("/")

    files = []
    for root, _, names in os.walk(a.src):
        for n in names:
            local = os.path.join(root, n)
            rel = os.path.relpath(local, a.src).replace(os.sep, "/")
            if not any(fnmatch.fnmatch(rel.lower(), p.lower()) for p in a.skip):
                files.append((rel, local, os.path.getsize(local)))
    files.sort()
    total = sum(t for _, _, t in files)
    print("%d files, %.2f GB -> %s:%d%s" % (len(files), total / 1e9, a.ip, a.port, dst), flush=True)

    f = connect(a.ip, a.port)
    made, sent, t0 = set(), 0, time.time()
    for i, (rel, local, size_of_set) in enumerate(files, 1):
        remote = dst + "/" + rel
        make_dirs(f, os.path.dirname(remote), made)
        try:
            if f.size(remote) == size_of_set:
                sent += size_of_set
                continue
        except ftplib.error_perm:
            pass
        for attempt in range(3):
            try:
                with open(local, "rb") as fh:
                    f.storbinary("STOR " + remote, fh, blocksize=1 << 20)
                if f.size(remote) != size_of_set:
                    raise IOError("size mismatch after upload")
                break
            except (OSError, EOFError, ftplib.Error) as e:
                print("  %s: %s, reconnecting" % (rel, e), flush=True)
                time.sleep(3)
                f = connect(a.ip, a.port)
        else:
            sys.exit("failed: " + rel)
        sent += size_of_set
        dt = max(time.time() - t0, 1e-3)
        print("[%d/%d] %s (%.0f MB) %.1f/%.1f GB %.1f MB/s" % (i, len(files), rel, size_of_set / 1e6, sent / 1e9,
              total / 1e9, sent / 1e6 / dt), flush=True)
    f.quit()
    print("done", flush=True)


if __name__ == "__main__":
    main()
