# Examples for runtime upgrade

Run example:

```
./runtime_upgrade_examples/test.sh <path-to-example>
```

for example:

```
./runtime_upgrade_examples/test.sh ./runtime_upgrade_examples/storage_upgrade/
```

`./runtime_upgrade_examples/test.sh` compiles initial version of ColdStack
pallet, then copy modified file and compiles WASM for new runtime.

`./runtime_upgrade_examples/storage_upgrade/` containes nontrivial upgrade that 
- adds method `test` and event `Test`
- Adds `Gateways` storage item to use instead of `GatewayNodeSeeds`. Then it
  moves all the data to new storage item in `on_runtime_upgrade` hook
 
Script `./runtime_upgrade_examples/storage_upgrade/test.js` performs runtime
upgrade and checks that upgrade is correct
