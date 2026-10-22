# datos_a_ftp.py ("data to FTP"): a developer's copy of a WoW client's Data folder to the console's FTP server, for
# tests on the Switch (the NRO reads sdmc:/wow-nx/Data). Players copy the folder to the SD card themselves.
#
#   python3 switch/datos_a_ftp.py <Data folder> <ip> [--destino /wow-nx/Data] [--puerto 5000] [--sin PATTERN]...
#
# Resumable: a remote file that already has the right size is skipped, so after an interruption run it again.
# Anonymous login, as the console's FTP servers use. --sin skips paths matching a glob (case-insensitive).
import argparse, fnmatch, ftplib, os, sys, time


def conectar(ip, puerto):
    f = ftplib.FTP()
    f.connect(ip, puerto, timeout=120)
    f.login("anonymous", "anonymous")
    f.voidcmd("TYPE I")
    return f


def crear(f, carpeta, hechas):
    partes = carpeta.strip("/").split("/")
    for i in range(1, len(partes) + 1):
        ruta = "/" + "/".join(partes[:i])
        if ruta not in hechas:
            try:
                f.mkd(ruta)
            except ftplib.error_perm:
                pass  # already there
            hechas.add(ruta)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("origen")
    ap.add_argument("ip")
    ap.add_argument("--destino", default="/wow-nx/Data")
    ap.add_argument("--puerto", type=int, default=5000)
    ap.add_argument("--sin", action="append", default=[])
    a = ap.parse_args()
    destino = "/" + a.destino.strip("/")

    ficheros = []
    for raiz, _, nombres in os.walk(a.origen):
        for n in nombres:
            local = os.path.join(raiz, n)
            rel = os.path.relpath(local, a.origen).replace(os.sep, "/")
            if not any(fnmatch.fnmatch(rel.lower(), p.lower()) for p in a.sin):
                ficheros.append((rel, local, os.path.getsize(local)))
    ficheros.sort()
    total = sum(t for _, _, t in ficheros)
    print("%d ficheros, %.2f GB -> %s:%d%s" % (len(ficheros), total / 1e9, a.ip, a.puerto, destino), flush=True)

    f = conectar(a.ip, a.puerto)
    hechas, enviado, t0 = set(), 0, time.time()
    for i, (rel, local, tam) in enumerate(ficheros, 1):
        remoto = destino + "/" + rel
        crear(f, os.path.dirname(remoto), hechas)
        try:
            if f.size(remoto) == tam:
                enviado += tam
                continue
        except ftplib.error_perm:
            pass
        for intento in range(3):
            try:
                with open(local, "rb") as fh:
                    f.storbinary("STOR " + remoto, fh, blocksize=1 << 20)
                if f.size(remoto) != tam:
                    raise IOError("size mismatch after upload")
                break
            except (OSError, EOFError, ftplib.Error) as e:
                print("  %s: %s, reconnecting" % (rel, e), flush=True)
                time.sleep(3)
                f = conectar(a.ip, a.puerto)
        else:
            sys.exit("failed: " + rel)
        enviado += tam
        dt = max(time.time() - t0, 1e-3)
        print("[%d/%d] %s (%.0f MB) %.1f/%.1f GB %.1f MB/s" % (i, len(ficheros), rel, tam / 1e6, enviado / 1e9,
              total / 1e9, enviado / 1e6 / dt), flush=True)
    f.quit()
    print("done", flush=True)


if __name__ == "__main__":
    main()
