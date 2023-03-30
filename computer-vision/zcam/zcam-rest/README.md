# zcam-rest -- Displaying video in a web browser through the zenoh webserver plugin
This is a simple web page able to display video from the RUST or Python zcam applications using zenoh webserver plugin.

## Running zcam-rest

### Run zenoh router with webserver plugin

```bash
$ zenohd -P webserver:/path/to/libzplugin_webserver.so --cfg "plugins/webserver:{http_port:8080,}"
```

### Run zcapture (RUST or Python)

- Rust
    ```bash
    $ zcapture
    ```
- Python
    ```bash
    $ python3 zcapture.py
    ```

### Open index.html

```bash
$ open index.html
```
