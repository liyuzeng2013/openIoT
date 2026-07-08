@echo off
md C:\Users\李钰锃\.cargo && echo. > C:\Users\李钰锃\.cargo\config
(
echo [source.crates-io]
echo registry = "https://github.com/rust-lang/crates.io-index"
echo replace-with = "ustc"
echo.
echo [source.ustc]
echo registry = "sparse+https://mirrors.ustc.edu.cn/crates.io-index/"
) > C:\Users\李钰锃\.cargo\config
type C:\Users\李钰锃\.cargo\config
pause