fn main(){
    cc::Build::new()
    .file("minicoro.c")
    .include("vendor/minicoro")
    .compile("minicoro")
}