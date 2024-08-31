# svgc

[![crates.io](https://img.shields.io/crates/v/svgc?style=for-the-badge)](https://crates.io/crates/svgc)

Версия утилиты [SvgCompress](https://github.com/pasabanov/SvgCompress/) на Rust.

## Описание

`svgc` — это инструмент для сжатия SVG-файлов путём удаления ненужных пробелов, комментариев, метаданных и некоторых других данных. Также поддерживается оптимизация с помощью [SVGO](https://github.com/svg/svgo) и сжатие в [SVGZ](https://ru.wikipedia.org/wiki/SVG#SVGZ). Утилита помогает уменьшить размер файла, очистить SVG-файлы для большей производительности и подготовить их к выпуску.

## Установка

#### Зависимости

Для установки или сборки утилиты необходимо установить [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html).

(Опционально) Если вы хотите использовать опцию `--svgo`, установите также [SVGO](https://github.com/svg/svgo).

### С помощью Cargo (рекомендуется):

```sh
cargo install svgc
```

### Из GitHub (самостоятельная сборка):

```sh
git clone https://github.com/pasabanov/svgc
cd svgc
cargo build --profile release
```

Собранный файл будет находиться в директории `target/release`.

## Использование

Чтобы сжать SVG-файлы, выполните скрипт с помощью следующей команды:

```sh
svgc [options] paths
```

## Опции

`-h`, `--help` Показать это сообщение и выйти  
`-v`, `--version` Показать версию программы  
`-r`, `--recursive` Обрабатывать директории рекурсивно  
`-f`, `--remove-fill` Удалить атрибуты `fill="..."`  
`-o`, `--svgo` Использовать SVGO, если он установлен в системе  
`-z`, `--svgz` Сжать в формат .svgz после оптимизации  
`-n`, `--no-default` Не выполнять оптимизаций по умолчанию (если вы хотите только использовать SVGO, сжать в .svgz или выполнить оба действия)  
`-q`, `--quiet` Выводить только сообщения об ошибках, не выводить обычные сообщения

## Примеры

1. Сжать один SVG-файл:
	```sh
	svgc my-icon.svg
	```
2. Сжать все SVG-файлы в указанных директориях и файлах:
	```sh
	svgc my-icons-directory1 my-icon.svg directory2 icon2.svg
	```
3. Сжать все SVG-файлы в директории и её поддиректориях:
	```sh
	svgc -r my-icons-directory
   ```
4. Сжать SVG-файл и удалить все атрибуты `fill="..."` (сделать картинку моноцветной):
	```sh
	svgc -f my-icon.svg
	```
5. Сжать все SVG-файлы в директории и её поддиректориях, удаляя атрибуты `fill`, затем оптимизировать с помощью SVGO, затем сжать в .svgz:
	```sh
	svgc -rfoz my-icons-directory
	```

## Лицензия

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.

## Авторские права

2024 Пётр Александрович Сабанов

## Метрики

![repo size](https://img.shields.io/github/repo-size/pasabanov/svgc?color=6e54bb)
![crate size](https://img.shields.io/crates/size/svgc?label=crate%20size&color=orange)