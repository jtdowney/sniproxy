# sniproxy

> [!WARNING]
> This is an incomplete toy

This project is a simple proxy server similar to the initial Tailscale sniproxy that was replaced with App Connectors. It starts a DNS server and returns the local address. It then proxies connections it receives to the destination sent via SNI. This will eventually break as Encrypted Client Hello (ECH) becomes more popular.

I am posting this code to save it for my future reference.