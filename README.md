## format-brackets

Базовое форматирование потоковых файлов.

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