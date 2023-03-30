# A zenoh plotting example

## Python random publisher

```bash
python3 z_pub_rand.py
```

## Python plot

```bash
python3 z_plot.py
```

## Web plot

Run a zenoh router:
```bash
zenohd
```

Open `z_plot.html`

## Freeboard plot

Run a zenoh router with a storage storing `demo/random`:

```bash
zenohd -P storage_manager --cfg 'plugins/storage_manager/storages/demo:{key_expr:"demo/random",volume:"memory",}'
```

Browse to http://freeboard.github.io/freeboard/ .

Click "LOAD FREEBOARD" and provide `dashboard.json` file.