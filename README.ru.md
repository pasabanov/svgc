# SvgCompress

Версия утилиты [SvgCompress](https://github.com/pasabanov/SvgCompress/) на Rust.

## Описание

`SvgCompress` — это инструмент для сжатия SVG-файлов путём удаления ненужных пробелов, комментариев, метаданных и некоторых других данных. Также поддерживается оптимизация с помощью [SVGO](https://github.com/svg/svgo) и сжатие в [SVGZ](https://ru.wikipedia.org/wiki/SVG#SVGZ). Утилита помогает уменьшить размер файла, очистить SVG-файлы для большей производительности и подготовить их к выпуску.

## Установка

1. **Клонирование репозитория:**

	```sh
	git clone https://github.com/pasabanov/SvgCompress-rs
	cd SvgCompress-rs
	```

2. **Сборка:**

	```sh
    cargo build --profile release
	```
 
	Собранный файл будет находиться в директории `target/release`.

3. **(Опционально) Если вы хотите использовать опцию `--svgo`, убедитесь, что [SVGO](https://github.com/svg/svgo) установлен.**

Обратите внимание, что утилита [gzip](https://www.gnu.org/software/gzip/) встроена в исполняемый файл и не требует установки в системе.

## Использование

Чтобы сжать SVG-файлы, выполните скрипт с помощью следующей команды:

```sh
SvgCompress-rs [options] paths
```

## Опции

`-h`, `--help` Показать это сообщение и выйти  
`-v`, `--version` Показать версию скрипта  
`-r`, `--recursive` Обрабатывать директории рекурсивно  
`-f`, `--remove-fill` Удалить атрибуты `fill="..."`   
`-o`, `--svgo` Использовать [SVGO](https://github.com/svg/svgo), если он установлен в системе  
`-z`, `--svgz` Сжать в формат [.svgz](https://ru.wikipedia.org/wiki/SVG#SVGZ) с помощью утилиты [gzip](https://www.gnu.org/software/gzip/) после обработки  
`-n`, `--no-default` Не выполнять оптимизаций по умолчанию (если вы хотите использовать только [SVGO](https://github.com/svg/svgo), [gzip](https://www.gnu.org/software/gzip/) или оба)

## Примеры
1. Сжать один SVG-файл:
	```sh
	SvgCompress-rs my-icon.svg
	```
2. Сжать все SVG-файлы в указанных директориях и файлах:
	```sh
	SvgCompress-rs my-icons-directory1 my-icon.svg directory2 icon2.svg
	```
3. Сжать все SVG-файлы в директории и её поддиректориях:
	```sh
	SvgCompress-rs -r my-icons-directory
   ```
4. Сжать SVG-файл и удалить все атрибуты `fill="..."` (сделать картинку моноцветной):
	```sh
	SvgCompress-rs -f my-icon.svg
	```
5. Сжать все SVG-файлы в директории и её поддиректориях, удаляя атрибуты `fill`, затем оптимизировать с помощью SVGO, затем сжать в .svgz с помощью gzip:
	```sh
	SvgCompress-rs -rfoz my-icons-directory
	```

## Лицензия

This project is licensed under the Creative Commons Attribution 4.0 International License (CC BY 4.0).

You are free to:
- Share — copy and redistribute the material in any medium or format
- Adapt — remix, transform, and build upon the material

Under the following terms:
- **Attribution** — You must give appropriate credit, provide a link to the license, and indicate if changes were made. You may do so in any reasonable manner, but not in any way that suggests the licensor endorses you or your use.

For more details, see the full license at https://creativecommons.org/licenses/by/4.0/

## Авторские права
2024 Пётр Александрович Сабанов