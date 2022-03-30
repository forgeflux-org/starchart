<div align="center">

# ForgeFlux StarChart

[![Documentation](https://img.shields.io/badge/docs-master-blue?style=flat-square)](https://forgeflux-org.github.io/starchart/starchart/)
[![Build](https://github.com/forgeflux-org/starchart/actions/workflows/linux.yml/badge.svg)](https://github.com/forgeflux-org/starchart/actions/workflows/linux.yml)
[![dependency status](https://deps.rs/repo/github/forgeflux-org/starchart/status.svg?style=flat-square)](https://deps.rs/repo/github/forgeflux-org/starchart)
[![codecov](https://codecov.io/gh/forgeflux-org/starchart/branch/master/graph/badge.svg?style=flat-square)](https://codecov.io/gh/forgeflux-org/starchart)
<br />
[![AGPL License](https://img.shields.io/badge/license-AGPL-blue.svg?style=flat-square)](http://www.gnu.org/licenses/agpl-3.0)
[![Chat](https://img.shields.io/badge/matrix-+forgefederation:matrix.batsense.net-purple?style=flat-square)](https://matrix.to/#/#forgefederation:matrix.batsense.net)

</div>

## Why

There are several small, private forges that host Free Software projects.
Some of these Forges might one day participate in the federated
ecosystem. So it would make sense to have a system(see
[spider mechanism](#consensual-spidering)) that would map and advertise these instances
and the projects that they host.

## Consensual Spidering

We are aware that spiders some [very
aggressive](https://git.sr.ht/~sircmpwn/sr.ht-nginx/commit/d8b0bd6aa514a23f5dd3c29168dac7f89f5b64e7)
and small forges are often running on resource-constrained environments.
Therefore, StarChart(this spider) will only crawl a service if the crawl is
requested by the admin of the forge(more accurately, folks that have
access to the DNS associated with the forge's hostname though).

StarChart will rate limit API calls to one call every 10 seconds. For
instance, a Gitea API call would resemble:

```bash
curl -X 'GET' \
  'https://gitea.example.org/api/v1/repos/search?page=2&limit=20' \
  -H 'accept: application/json'
```

## Contributing

Thanks for considering contributing on GitHub. If you are not an GitHub
but would like to contribute to ForgeFlux sub-projects(all repositories
under this organisation), I would be happy to manually mirror this
repository on my [Gitea instance](https://git.batsense.net), which has a
much [more respectful privacy policy](https://batsense.net/privacy-policy)
