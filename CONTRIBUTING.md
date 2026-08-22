# Contributing

QuantForge is a local-first research workbench. PRs should make a long-term
investor more able to open a name, judge quality / cheapness / survival, write
the call, and keep a short list.

## Run

```sh
make start
```

Open [http://127.0.0.1:4176](http://127.0.0.1:4176). `make dev` is desk HMR on
4177 with the host on 4176.

## Check

```sh
make check
```

## Rules

- Bind loopback only. Do not serve on `0.0.0.0`.
- Persist in QuantForge SQLite. Do not share other product databases.
- Provider keys stay in the host store. Settings may say whether a key exists,
  never the key.
- Do not add a tape, a broker, a screener, or a hosted product.
- Family monochrome tokens only. Charts use weight and dash, not hue.
- Refresh is explicit or first-open.
