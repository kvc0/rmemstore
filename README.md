# rmemstore

Fast, type-aware data structure cache.

# About
`rmemstore` is similar to other caches you may have used, like redis, but it has some different goals than most.
The primary aims of `rmemstore` is to be typesafe, fast, and useful.

Of course, usefulness is an ongoing exercise, as it takes time to grow features. However, `rmemstore` is a type-
aware data structure store, which means you can store maps of maps - and the server knows what that means.

It is fast now, however. `rmemstore` uses the new Sieve eviction strategy when pressed to eviction. With a 10:1
read:write ratio, 2 threads on my 11 year old Intel i5 server are capable of over 3.3 million operations per
second. Even while being pushed to eviction.

`rmemstore` is built on "safe" Rust code. It doesn't rely on subtle tricks to get speed. It does use standard
libraries like the excellent `tokio` which may use dark magic, but they're trustworthy.

`rmemstore` uses bare tcp - no application frameworks. Each 0 and every 1 that your network card transmits to or
from an `rmemstored` server has a direct purpose. Inventing a new ostensibly-portable wire protocol is a vaguely
hubric exercise when suitable alternatives exist. With that in mind, `rmemstore` uses `protosockets`, which is a
compromise between the aforementioned hubris and pragmatism.

# Protocol

The tcp stream inbound to `rmemstored` is a stream of standard, length-delimited protocol buffers `rmemstore.Rpc`
structures. These messages carry an id, and `rmemstored` responds with that id - possibly out of order. It is a
multithreaded, multiplexing server. You can send as much as you want as fast as you can, subject to your network and
cpu capabilities.

The tcp stream outbound from `rmemstored` is a stream of standard, length-delimited protocol buffers `rmemstore.Response`
structures. These messages carry the id from the Rpc that initiated the response. Every `rmemstore.Rpc` has a
corresponding `rmemstore.Response`.

Inbound and outbound streams are: `varint` `message` `varint` `message`[...]. The varint before the message is the
length of the message. So once you have read the bytes for `varint` and the length of `varint`, you have a complete
message.

# Languages
## Rust
You can look at [`rmem`](./rmem/src/main.rs) for an example of how you can use the client. Usage boils down to 3
lines:
```rust
let mut configuration = rmemstore::ClientConfiguration::new();
let client = configuration.connect(args.host.to_string()).await?;
client.put("some key", "some value").await?;
```
You can also put dictionaries:
```rust
client.put(
    "some key",
    HashMap::<&str, &str>::from_iter([
        ("hello", "world")
    ]),
).await?;
```
or dictionaries of strings and dictionaries, however wild you want to get:
```rust
client
    .put(
        "some key",
        HashMap::<&str, MemstoreValue>::from_iter([
            (
                "hello",
                MemstoreValue::String {
                    string: "world".to_string(),
                },
            ),
            (
                "nested",
                MemstoreValue::Map {
                    map: HashMap::from_iter([(
                        "inner".to_string(),
                        MemstoreValue::String {
                            string: "values".to_string(),
                        },
                    )]),
                },
            ),
        ]),
    )
    .await?;
```
## Bash
You can use `rmem` to put and get.

For strings, the output is a little more brief.
```bash
$ rmem put foo `{"string": "some value"}`
```

```bash
$ rmem get foo
some value
```

For maps, the interaction has some verbosity, but it is typed!

```bash
$ rmem put foo '{"map": {"bar":{"map":{"baz":{"string": "haha"}, "other": {"string": "verbose"}}, "outer": {"string": "another"}}}}'
```

```
$ rmem get foo
{
  "bar": {
    "map": {
      "baz": {
        "string": "haha"
      },
      "other": {
        "string": "verbose"
      }
    }
  }
}
```

## Python
Don't want to use rust? Any tool or language capable of sending and receiving protocol buffers-encoded bytes over
tcp is capable of using `rmemstored`. See [`example-python`](./example-python/main.py) for an example in another
language. Note that python, in particular, is a bit of a pain due to not exposing the protobuf varint encoder.
