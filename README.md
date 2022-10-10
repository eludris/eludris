<p align="center"><img width="400em" src="https://github.com/eludris/.github/blob/main/assets/das_ding.png" /></p>

# Eludris

A free and open source, federated, End-To-End-Encrypted social media platform made in rust that's easy to deploy and configure
while striving to be *truly **yours***.

Eludris tries to combine the best parts of other popular social media platforms such as Discord, Reddit, Twitter and so on while
not being one or the other.

## Deployment

We really recommend and *only* officially support using the provided docker-compose as a quick way to get stuff running, just
edit your `Eludris.toml` to suit your needs then run

```sh
docker-compose up
```

Congratulations, you've now successfully deployed your Eludris instance! <img width="30em" src="https://github.com/eludris/.github/blob/main/assets/thang-big.png" />

## Default Ports

[Oprish](https://github.com/eludris/oprish) (HTTP API): 7159

[Pandemonium](https://github.com/eludris/pandemonium) (WS API/ Gateway): 7160

[Effis](https://github.com/eludris/effis) (File server, CDN and proxy): 7161
