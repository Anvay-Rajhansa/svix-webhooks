<p align="center">
  <img width="120" src="https://avatars.githubusercontent.com/u/80175132?s=200&v=4" />
  <h1 align="center">Svix - Webhooks as a service</h1>
</p>

Libraries for interacting with the Svix API and verifying webhook signatures

# Documentation

The docs are available at https://docs.diahook.com

# Structure

Each subdirectory has a library for a different language.
The code is a combination of code auto-generated from the OpenAPI spec, and manually written wrappers.

# Building

```
# Install deps
yarn
./regen_openapi.sh https://api.diahook.com/api/v1/openapi.json
```

Followed by running the build for each of the libraries (subdirectories).
