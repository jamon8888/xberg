import Config

# Force building Rustler NIF from source when using local path dependency
config :rustler_precompiled,
  force_build: [kreuzberg: true]
