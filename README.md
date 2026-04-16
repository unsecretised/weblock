# Weblock

Add a password gateway to any website through this simple http rev proxy

## Installation:

### Via cargo:

`cargo install weblock`

### Build it yourself (requires rust):
```
git clone https://github.com/unsecretised/weblock

cd weblock

cargo install --path .
```

## Regular usage:

`weblock tunnel_name create -i 8080 -o 8000 -p SomePassword`

-i or -inport means the port where the proxy will be hosted

-o or -outport is the port to proxy the requests to

-p or -password is the password you want to use


## Programmatic usage:

To allow access to the service via code, you can either provide a cookie header:
```
Cookie: weblock_auth={{ JWT FROM BROWSER DEV TOOLs }}
```

or a weblock_auth header:

```
weblock_auth={{ JWT FROM BROWSER DEV TOOLS }}
```


