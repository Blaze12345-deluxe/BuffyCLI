VERSION = "2026.07.27"
AUTHOR = "Buffy Community"
DESCRIPTION = "Runs network diagnostics: ping and traceroute."
OUTPUT = false

// Usage: buffy --run network-diagnostic.bsl <hostname>
// Example: buffy --run network-diagnostic.bsl google.com

// Set default target
WRITE "Network Diagnostic Tool"
WRITE ""

// Check if a target was provided
WRITE "Target: ${1}"
WRITE ""

WRITE "========================================="
WRITE "  Step 1: DNS Resolution"
WRITE "========================================="

OUTPUT = true
RUN "nslookup ${1} 2>/dev/null || host ${1} 2>/dev/null || echo 'DNS lookup tools not available'"

OUTPUT = false
WRITE ""
WRITE "========================================="
WRITE "  Step 2: Ping Test"
WRITE "========================================="

OUTPUT = true
RUN "ping -c 4 ${1} 2>/dev/null || echo 'Ping failed or not available'"

OUTPUT = false
WRITE ""
WRITE "========================================="
WRITE "  Step 3: Traceroute"
WRITE "========================================="

OUTPUT = true
RUN "traceroute ${1} 2>/dev/null || tracert ${1} 2>/dev/null || echo 'Traceroute not available'"

OUTPUT = false
WRITE ""
WRITE "========================================="
WRITE "  Step 4: Connection Test"
WRITE "========================================="

OUTPUT = true
RUN "curl -sI https://${1} 2>/dev/null | head -5 || wget --spider https://${1} 2>/dev/null || echo 'Connection check tools not available'"

OUTPUT = false
WRITE ""
WRITE "========================================="
WRITE "  Diagnostic Complete"
WRITE "========================================="

EXIT
