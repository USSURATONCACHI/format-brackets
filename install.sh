#!/bin/bash -e

cargo build --release
cp ./target/release/format-brackets ~/Системное/Утилиты/
cd ~/Системное/ && ./ОбновитьСимлинки
