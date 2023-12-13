# zcam-rest -- Displaying video in a web browser through the zenoh webserver plugin

This is a simple web page able to display video from the RUST or Python zcam applications using zenoh webserver plugin.

## Running zcam-rest

### Run zenoh router with webserver plugin

Install the [WebServer](https://github.com/eclipse-zenoh/zenoh-plugin-webserver)
plugin under `~/.zenoh/lib`, or specify another directory through `--plugin-search-dir`.

```bash
zenohd --cfg 'plugins/webserver/http_port:8080'
```

### Run zcapture (Rust or Python)

- Rust

    ```bash
    zcapture
    ```

- Python

    ```bash
    python3 zcapture.py
    ```

### Open index.html

```bash
open index.html
```
