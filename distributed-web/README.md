# Distributed-web demos

### Building a website from geo-dristributed content

$USE and $USW are two different host names, in our case scenario the two host are:

$USE = us-east.zenoh.io
$USW = us-west.zenoh.io

# Create a filesystem backend
```
curl -X PUT -H 'content-type:application/properties' -d "" $USW/@/router/local/plugin/storages/backend/fs
```
<!-- sleep 0.5 -->

# Create a filesystem storage on /public/**

```
curl -X PUT -H 'content-type:application/properties' -d "path_expr=/public/**;path_prefix=/public;dir=public;read_only" $USW/@/router/local/plugin/storages/backend/fs/storage/public
```

<!-- # On 1st working station: i.e. us-east.zenoh.io -->
# Filesystem backend

```
curl -X PUT -H 'content-type:application/properties' -d "" $USE/@/router/local/plugin/storages/backend/fs
```

<!-- sleep 0.5 -->
# Filesystem storage on /public/**

```
curl -X PUT -H 'content-type:application/properties' -d "path_expr=/public/**;path_prefix=/public;dir=public;read_only" $USE/@/router/local/plugin/storages/backend/fs/storage/public
```