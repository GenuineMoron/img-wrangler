# Security

Wrangler opens files you didn't make. A broken image that crashes the decoder, writes outside the
folder it should, or slips something past the undo store is a security problem, and I'd like to hear
about it privately first.

## Reporting

Use **Report a vulnerability** on the repository's [Security tab](https://github.com/GenuineMoron/img-wrangler/security). It reaches me without going public. Include the file that triggers it if you can, the version, and your operating system.

You'll get a reply within a week. Once there's a fix, the advisory goes public with credit to you,
unless you'd rather not be named.

## What's covered

The engine, the command line tool, the desktop app and the right-click integrations, on the
latest release and `main`. Wrangler makes no network requests, so most of the risk is in decoding
and in where files get written.
