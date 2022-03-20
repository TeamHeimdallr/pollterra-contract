# Optimized builds
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  greengaguri/workspace-optimizer:0.12.5-protoc

# If you encounter the error message related to protoc,
# it can be caused by the old volumes made by cosmwasm/workspace-optimizer:0.12.5,
# so retry this script after cleaning up the volumes;pollterra-token_cache, registry_cache
