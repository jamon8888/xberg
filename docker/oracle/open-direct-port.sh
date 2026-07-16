#!/usr/bin/env bash
# Optional — only needed if you skip the Cloudflare Tunnel and expose the
# mcp-server port directly instead (e.g. to terminate TLS yourself).
#
# With the tunnel-based setup (docker-compose.yml's cloudflared service) this
# script is NOT needed: the tunnel makes an outbound-only connection, so no
# inbound port beyond the SSH one OCI already opens for you has to change.
#
# OCI Ubuntu images enforce a default-deny on inbound traffic via iptables
# directly (not ufw — leave ufw disabled, OCI docs warn enabling it can
# prevent the instance from booting). The rule must be inserted BEFORE the
# existing REJECT rule or it is silently ignored; appending to the end of the
# file does not work.
#
# Usage: sudo ./open-direct-port.sh 8080
set -euo pipefail

PORT="${1:?usage: open-direct-port.sh <port>}"
RULES_FILE=/etc/iptables/rules.v4

if [[ ! -f "$RULES_FILE" ]]; then
  echo "error: $RULES_FILE not found — is this an OCI Ubuntu image?" >&2
  exit 1
fi

if grep -q -- "--dport ${PORT} -j ACCEPT" "$RULES_FILE"; then
  echo "Port ${PORT} already allowed in ${RULES_FILE}, nothing to do."
  exit 0
fi

cp "$RULES_FILE" "${RULES_FILE}.bak.$(date +%s)"

# Insert immediately before the first REJECT line so the ACCEPT rule is
# evaluated first (iptables is first-match-wins).
sudo sed -i "0,/-j REJECT/s//-A INPUT -p tcp -m state --state NEW -m tcp --dport ${PORT} -j ACCEPT\n&/" "$RULES_FILE"

sudo netfilter-persistent reload
echo "Opened inbound TCP ${PORT}. Backup saved as ${RULES_FILE}.bak.*"
