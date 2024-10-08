# A zenoh plotting example

## Python random publisher

```bash
python3 z_pub_rand.py
```

## Python plot

```bash
python3 z_plot.py
```

## SSE plot

Run a zenoh router:

```bash
zenohd
```

Open `z_plot.html`

## TypeScript plot

Run a zenoh router with remote api plugin:

```bash
zenohd --cfg plugins/remote_api/websocket_port:10000
```

Run z_plot_ts:

```bash
cd z_plot_ts
npm install
npm run dev
```

Browse to <http://localhost:5173/>

## Freeboard plot

Run a zenoh router with a storage storing `demo/random`:

```bash
zenohd -P storage_manager --cfg 'plugins/storage_manager/storages/demo:{key_expr:"demo/random",volume:"memory",}'
```

Browse to <https://freeboard.github.io/freeboard/>.

Click "LOAD FREEBOARD" and provide `dashboard.json` file.
