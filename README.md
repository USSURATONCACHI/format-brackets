## format-brackets

Базовое форматирование потоковых файлов.
На данный момент не самое красивое форматирование, но рабочее. В процессе работы внешний вид будет улучшаться.

Можно добавлять свои правила форматирования (шаблоны скобок и литералов) через аргументы командной строки.
Больше информации по команде `$ cargo run -- --help`.

```bash
$ cat hello_world.c | cargo run
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.08s
     Running `target/debug/format-brackets`
```
```c
#include <stdio.h>

int world(
) {

    int a = 1 + (
        2 + 3
    );
    printf(
        "a = %d", a
    );
}

// \( this should be ignored

int main(
) {

    printf(
        "Hello, world()!\n"
    );
    world(
    );
}
```

### TODO:
- Не ставить новые строки между скобками со слишком маленьким количеством символов между ними 
- Понятие запятых/делиметров списков
- Токенайзер-вывод для интеграции с другими форматтерами
- Именованные файлы-конфиги (/etc/format-brackets/, ~/.config/format-brackets/)