# Mini ADAS Example

Example of a minimal dummy ADAS activity set.

## Running

You need to run the following commands in separate terminals.

```sh
# Use 400ms cycle time
bazelisk run //examples/rust/mini-adas:adas_primary_com_iox2_direct_unix -- 400
```

```sh
bazelisk run //examples/rust/mini-adas:adas_secondary_com_iox2_direct_unix -- 1
```

```sh
bazelisk run //examples/rust/mini-adas:adas_secondary_com_iox2_direct_unix -- 2
```

It's possible to switch between com backend implementations:
* com_iox2 for Iceoryx2
* com_linux_shm for Linux shared memory backend
* com_mw for middleware COM backend

And signalling implementations:
* direct_tcp for direct connections via TCP sockets
* direct_unix for direct connections via UNIX sockets
* relayed_tcp for relayed signalling via TCP sockets
* relayed_unix for relayed signalling via UNIX sockets
* direct_mw_com for direct connections via middleware COM (WIP, not available yet)

For instance, Linux shared memory com backend with TCP scokets signalling implementation may be started with:

```sh
# Use 400ms cycle time
bazelisk run //examples/rust/mini-adas:adas_primary_com_mw_direct_mw_com -- 400
```

```sh
bazelisk run //examples/rust/mini-adas:adas_secondary_com_mw_direct_mw_com -- 1
```

```sh
bazelisk run //examples/rust/mini-adas:adas_secondary_com_mw_direct_mw_com -- 2
```

## Different signalling layer

The easiest way to switch the signalling layer is by changing the crate_features in the `BUILD.bazel`,
make sure to switch it for every target you're using. Then you can just use the commands from above.

Note that for mpsc-only signalling, there can be only a primary process without
any secondaries, because mpsc does not support inter-process signalling.

## Running tracer

In order to start tracing use:

```sh
bazel run //src/feo-tracer:feo_tracer -- -o out.dat
```
where `out.dat` is the tracing data output.

You can specify the tracing duration in seconds and log level using:

```sh
bazel run //src/feo-tracer:feo_tracer -- -d 10 -l INFO -o out.dat
```
