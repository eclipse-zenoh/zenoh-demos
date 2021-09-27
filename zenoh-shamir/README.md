# Leveraging Zenoh for Shamir’s Secret Sharing Scheme

For the unfamiliar reader, [Shamir’s Secret Sharing Scheme](https://en.wikipedia.org/wiki/Shamir%27s_Secret_Sharing) is a way to protect a secret by "splitting" it into *shares*. To then reconstruct the secret, one needs to access *enough* of these shares. Indeed, not necessarily all of them are required: this scheme is known as a `(k, n)-threshold` scheme, where `k` shares among the `n` produced are needed to recover the secret.

The objective of this demonstration is to show how we can use Zenoh to store the shares, retrieve (enough of) them and finally reconstruct the secret.


## Requirements

* A [Rust](https://rustup.rs/) environment.
* Several [Zenoh routers](https://zenoh.io/docs/getting-started/quick-test/) — we will see below how to start them.
* A way to make a `GET` call on a Zenoh router — [curl](https://github.com/eclipse-zenoh/zenoh#putget-into-zenoh) or, even better, the [z_get](https://github.com/eclipse-zenoh/zenoh/tree/master/zenoh/examples/zenoh#z_get) Zenoh example.


## Usage

### How to build

Launch the following command inside this folder:

```sh
cargo build
```

It will create two executables: `./target/debug/zenoh_put_shamir` and `./target/debug/zenoh_eval_shamir`.


### How to run

Before launching the routers, a quick explanation is required.

When called, `./target/debug/zenoh_put_shamir` will create shares of the value that is passed. The number of shares is controlled by the parameters `--threshold (-t)` and `--redundancy (-r)`, which respectively represent the number of shares required to recover the secret and the number of "copies" each share should have (the term "copy" is incorrect but the idea is similar).

The shares are put to `/share/{{i}}/{{path}}` where `{{i}}` is the index of the share and `{{path}}` the path provided when calling the executable.

For instance, if the following call is made:

```sh
./target/debug/zenoh_put_shamir -p "/demo/secret" -v "s3cr3t" -t 2 -r 2
```

What happens is:
* 4 shares are created (2 × 2);
* they are put to:
  * `/share/0/demo/secret`,
  * `/share/1/demo/secret`,
  * `/share/2/demo/secret`,
  * `/share/3/demo/secret`.

This means that we need subscribers for the paths `/share/0/**`, …, `/share/3/**`. Note that, for now, it is not possible to modify the base path (`/share/{{i}}`) as it was hardcoded in both executables.

With just this setup, it is possible to retrieve the shares and manually reconstruct the secret. But, as we can easily automate that part with an eval, this is exactly what `./target/debug/zenoh_eval_shamir` does: it tries to fetch enough shares in order to reconstruct the secret.

#### Starting the router(s)

For the purpose of having a distributed setting, we will consider that `threshold` and `redundancy` are both equal to 2 (the default values if the parameters are omitted) and thus start 4 routers, each in its own terminal:

```sh
./target/debug/zenohd -l tcp/127.0.0.1:7447 --rest-http-port 8000 --mem-storage="/share/0/**"

./target/debug/zenohd -l tcp/127.0.0.1:7448 --rest-http-port 8001 --mem-storage="/share/1/**" \
    -e tcp/127.0.0.1:7447

./target/debug/zenohd -l tcp/127.0.0.1:7449 --rest-http-port 8002 --mem-storage="/share/2/**" \
    -e tcp/127.0.0.1:7447 -e tcp/127.0.0.1:7448

./target/debug/zenohd -l tcp/127.0.0.1:7450 --rest-http-port 8003 --mem-storage="/share/3/**" \
    -e tcp/127.0.0.1:7447 -e tcp/127.0.0.1:7448 -e tcp/127.0.0.1:7449
```

#### PUT a secret

```
./target/debug/zenoh_put_shamir -p "/demo/secret" -v "s3cr3t" -t 2 -r 2
```

#### Registering the Eval

```
./target/debug/zenoh_eval_shamir -p "/shamir" -t 2 -r 2
```

#### GET the secret back

* Via `curl`:
  ```
  curl "http://localhost:8000/shamir?(name=/demo/secret)"
  ```
  Which should return:
  ```json
  { "key": "/shamir", "value": "s3cr3t", "encoding": "text/plain", "time": "2021-05-07T09:06:17.707710000Z/00" }
  ```
  

* Via `z_get`:
  ```
  ./target/debug/examples/z_get -s "/shamir?(name=/demo/secret)"
  ```
  Which should return:
  ```
  /shamir : StringUtf8("s3cr3t") (encoding: text/plain , timestamp: 2021-05-07T09:04:05.506096000Z/00)
  ```
  
And voilà! With this small demonstration we managed to create shares of a value, store them and finally get them back in order to recover our value, all while using Zenoh.

Note that it is entirely possible to query the shares separately, as they are valid named resources. For instance, to get the first share:

```
curl "http://localhost:8000/share/0/demo/secret"
```

```json
{ "key": "/share/0/demo/secret", "value": "Af/toS5UZA==", "encoding": "application/octet-stream", "time": "2021-05-07T09:01:57.717134997Z/B77328F198C34C48B1056216AA3CF157" }
```
