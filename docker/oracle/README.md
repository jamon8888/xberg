# Deploying xberg-mcp-server on Oracle Cloud Always Free

Runs the TypeScript `mcp-server` (RAG + extraction + PII redaction, serving
the static web UI at `/ui`) on an Oracle Cloud **Ampere A1** Always Free VM,
reachable through a **Cloudflare Tunnel** (no inbound ports beyond SSH).

## 1. Provision the VM

1. Oracle Cloud Console → Compute → Instances → Create instance.
2. Shape: **VM.Standard.A1.Flex**, Always Free eligible — 2 OCPU / 12 GB RAM.
3. Image: **Ubuntu 24.04** (aarch64/ARM).
4. Boot volume: default is fine (Always Free gives ~200 GB total across boot
   + block volumes).
5. Add your SSH key. Networking: default VCN/subnet is fine — the security
   list only needs to allow inbound 22 (already the default).

If you hit **"out of host capacity"**: this is common for Ampere A1, not a
config mistake. Retry across availability domains in your home region, or
retry over a few hours/days. Your Always Free allowance doesn't disappear.

## 2. Base setup (SSH in)

```sh
sudo apt update && sudo apt install -y docker.io docker-compose-v2 git
sudo usermod -aG docker "$USER"   # log out/in once for this to take effect
sudo mkdir -p /opt/xberg && sudo chown "$USER" /opt/xberg
git clone https://github.com/xberg-io/xberg /opt/xberg   # or your fork's URL
```

Leave `ufw` disabled — OCI Ubuntu images manage the host firewall directly
via iptables, and enabling ufw on top of that is documented by Oracle to
risk the instance not booting.

## 3. Set up the Cloudflare Tunnel

1. In the Cloudflare Zero Trust dashboard: **Networks → Tunnels → Create a
   tunnel** → connector type **Cloudflared**.
2. Copy the token shown in the install step (you won't run the install
   command it suggests — `docker-compose.yml`'s `cloudflared` service uses
   the token directly).
3. Add a **Public Hostname**: your domain (e.g. `mcp.yourdomain.com`) →
   service `http://mcp-server:8080`.

This gives you HTTPS, DDoS protection, and no open inbound port for the app
at all — only outbound from `cloudflared` to Cloudflare's edge.

## 4. Configure and start

```sh
cd /opt/xberg/docker/oracle
cp .env.example .env
# Fill in XBERG_MCP_UI_TOKEN (openssl rand -hex 32) and CLOUDFLARE_TUNNEL_TOKEN
docker compose build   # first build compiles ONNX Runtime + tesseract from
                        # source — expect this to take a while
docker compose up -d
docker compose logs -f mcp-server   # watch for "[xberg-mcp] started"
```

Then enable the systemd unit so it survives reboots:

```sh
sudo cp xberg-mcp.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now xberg-mcp.service
```

## 5. Verify

```sh
curl https://mcp.yourdomain.com/health
# {"status":"ok","server":"xberg-mcp"}
```

Open `https://mcp.yourdomain.com/ui?token=<your XBERG_MCP_UI_TOKEN>` for the
web UI. Point an MCP client (Claude Desktop, Cursor, etc.) at
`https://mcp.yourdomain.com/sse?token=<token>`.

## Known limitations, on purpose left as-is

- **Token is a shared static secret**, not per-client OAuth — fine for a
  personal/small-team deployment, not for exposing this to the public
  internet at large. Pinning `XBERG_MCP_UI_TOKEN` (step 4) avoids it
  rotating on every restart, but rotating it manually still means updating
  every client's saved URL.
- **First build is slow** — ONNX Runtime, tesseract, and libheif compile
  from source in the Alpine builder stage (`docker/Dockerfile.mcp-server`).
  This is the same tradeoff `docker/Dockerfile.musl-node` already makes in
  CI; subsequent builds reuse Docker layer caching unless crate source
  changes.
- **The `xberg-rag-node` and wasm builder stages are new** (unlike the
  `xberg-node` stage, which mirrors already-proven CI). If the first build
  fails, it's most likely a missing crate copy or a feature-flag mismatch —
  check the error against `crates/xberg-rag-node/Cargo.toml`'s declared
  features, not an architectural problem with this approach.
- **No automated backups.** The RAG store (`/data/store/store.sqlite3`) and
  PII rehydration maps (`/data/cache/rehydration/`) live only in the
  `xberg-data` Docker volume. Consider periodically copying them to Oracle
  Object Storage (Always Free includes a small allowance) if the data
  matters beyond casual use.

## Updating

```sh
cd /opt/xberg && git pull
cd docker/oracle && docker compose build && docker compose up -d
```
