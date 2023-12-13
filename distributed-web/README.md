# Distributed-web demo

This demo describes a simple static website whose content is geo-dristributed on
two Zenoh nodes.

## Prerequisites

- Install [zenohd](https://zenoh.io/docs/getting-started/installation/) (the Zenoh router) somewhere in your PATH.
- Install the [Web
  Server](https://github.com/eclipse-zenoh/zenoh-plugin-webserver) plugin and
  the [Filesystem](https://github.com/eclipse-zenoh/zenoh-backend-filesystem)
  backend of the Storage Manager plugin (comes with `zenohd`) under `~/.zenoh/lib`.

## Setup

First, two routers are deployed on (conceptually) geo-dristributed hosts; although for
the purposes of this demo we run both on `localhost`. These routers can be
configured through the [REST API](https://zenoh.io/docs/apis/rest/) on ports
`800{0,1}` and they run the Web Server plugin on ports
`808{0,1}`. Dynamic configuration is only possible when setting adminspace
permissions to read/write. The Zenoh endpoints are `tcp/localhost:744{7,8}`.

```bash
zenohd --adminspace-permissions rw -P storage_manager --cfg 'plugins/webserver/http_port:8080' --rest-http-port 8000 -l tcp/localhost:7447
zenohd --adminspace-permissions rw -P storage_manager --cfg 'plugins/webserver/http_port:8081' --rest-http-port 8001 -l tcp/localhost:7448 -e tcp/localhost:7447
```

Next, add an `fs` backend on each host:

```bash
curl -X PUT -H 'content-type:application/json' -d '{}' localhost:8000/@/router/local/config/plugins/storage_manager/volumes/fs
curl -X PUT -H 'content-type:application/json' -d '{}' localhost:8001/@/router/local/config/plugins/storage_manager/volumes/fs
```

The Zenoh Filesystem backend will use `~/.zenoh/zenoh_backend_fs` as a root
directory for the storages that we will create subsequently. This can be
customized through the `ZENOH_BACKEND_FS_ROOT` environment variable.

Note that Zenoh routers can be dynamically configured only through the REST API
and not the `webserver` plugin.

Next, create a storage under the key expression `/public/**` in each of the
hosts using the `fs` backend:

```bash
curl -X PUT -H 'content-type:application/json' -d '{key_expr:"public/**",volume:{id:"fs",dir:"foo"}}' localhost:8000/@/router/local/config/plugins/storage_manager/storages/foo
curl -X PUT -H 'content-type:application/json' -d '{key_expr:"public/**",volume:{id:"fs",dir:"bar"}}' localhost:8001/@/router/local/config/plugins/storage_manager/storages/bar
```

## Deploy

Our web page is composed of three files: `index.html`, `style.css` and
`zenoh-dragon.png` which we distribute across the storages `foo` and `bar` (assming that `ZENOH_BACKEND_FS_ROOT` is set):

```bash
mkdir -p $ZENOH_BACKEND_FS_ROOT/foo/public && cp index.html style.css $ZENOH_BACKEND_FS_ROOT/foo/public
mkdir -p $ZENOH_BACKEND_FS_ROOT/bar/public && cp zenoh-dragon.png $ZENOH_BACKEND_FS_ROOT/bar/public
```

The web page can now be accessed either at `localhost:8080/public` or at `localhost:8081/public`.
