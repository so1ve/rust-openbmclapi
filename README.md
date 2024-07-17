# rust-openbmclapi

Rust port of [openbmclapi](https://github.com/bangbang93/openbmclapi)

## Configuration

Please copy the [config example](./config.example.toml) and rename it to `config.toml`.

## Logging

Logging is done using the `tracing` crate. You can set the log level by setting the `RUST_LOG` environment variable. For example, to set the log level to `trace`, you can run the following command:

```sh
$ RUST_LOG=rust_openbmclapi=TRACE cargo run
```

## 📝 License

[MIT](./LICENSE). Made with ❤️ by [Ray](https://github.com/so1ve)
