

```shell
llvm-strip ./target/x86_64-unknown-linux-musl/release/static_package
```


```shell
upx --strip-relocs=0 --best -f -o ./target/x86_64-unknown-linux-musl/release/static_package_upx ./target/x86_64-unknown-linux-musl/release/static_package
```


```shell
docker build . -t <自定义>:<自定义>
```
