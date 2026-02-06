

```shell
llvm-strip ./target/x86_64-unknown-linux-musl/release/static_package
```


```shell
upx --strip-relocs=0 --best -f -o ./target/x86_64-unknown-linux-musl/release/static_package_upx ./target/x86_64-unknown-linux-musl/release/static_package
```


```shell
docker build . -t <自定义>:<自定义>
```




# 引入方式

前端项目打包,使用docker 基于本镜像将静态资源打包成镜像



## 构建

```shell
RUSTC_WRAPPER="" cross build --target x86_64-unknown-linux-musl 

```