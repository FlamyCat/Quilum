# Заметки о блокировке

## Общие примечания

Quilum не определяет автоматически, какие приложения считать отвлекающими. Их
список определяется пользователем.

Под блокировкой отвлекающего приложения понимается приведение системы в такое
состояние, при котором запуск или продолжительное (в том числе в течение
нескольких секунд) использование отвлекающего приложения пользователем
невозможно или максимально сильно затруднено.

Методы, описанные здесь, разделены на две категории:

- превентивные - те, которые в принципе не дают запускать отвлекающее приложение;
- реактивные - те, которые подразумевают завершение работы отвлекающего
  приложения в момент его запуска.

Поскольку применение любого из этих методов в какой-то степени нарушает
нормальную работу системы (пользователь не может запустить определенные
приложения, несмотря на то, что у него изначально были на это права), методы
анализируются с точки зрения деструктивности - сложности приведения системы в
полностью рабочее состояние в случае неполадок в работе Quilum и/или его
вспомогательных компонентов (например, демона, о котором речь пойдет в разделе,
посвященном Linux).

Метод называется fail-safe, если в случае неполадок пользователю не нужно ничего
предпринимать, чтобы пользоваться системой, как обычно.

Примеры кода составлены Claude Opus 4.6, с незначительными правками от меня.

## Windows

### Слой 1: SetWinEventHook — реактивное обнаружение окон

#### Описание

Метод подразумевает подписку на оконные события.

Windows предоставляет механизм Accessibility Event Hooks, что позволяет
выполнять произвольные действия (в данном случае - завершение отвлекающего
приложения) при создании окна.

Метод не подразумевает активное опрашивание чего-либо, а реакция на события
практически моментальная.

_Алгоритм:_

```
Событие: EVENT_SYSTEM_FOREGROUND (или EVENT_OBJECT_CREATE)
    │
    ▼
Получить HWND окна из параметров callback
    │
    ▼
GetWindowThreadProcessId(hwnd) → получить PID
    │
    ▼
OpenProcess(pid) → QueryFullProcessImageNameW()
    → получить полный путь к .exe
    │
    ▼
Путь есть в списке заблокированных?
    │
    ├── НЕТ → ничего не делаем
    │
    └── ДА →
        ├── TerminateProcess(handle) — убить процесс
        │   (или)
        └── PostMessage(hwnd, WM_CLOSE) — вежливо попросить закрыться
```

#### Детали реализации

Для реализации можно использовать крейты
[win_event_hook](https://docs.rs/win_event_hook) или [wineventhook](https://docs.rs/wineventhook).

Первый крейт использует синхронный механизм работы, а второй - асинхронный.

Первый, похоже, сильно проще в использовании, чем второй, поэтому
предпочтительнее, поскольку в остальном они очень похожи. К тому же, первый
позволяет снимать хук автоматически с помощью `Drop`.

Однако автор второго крейта заявляет, что реализация `Drop` для автоматического
снятия хука [не требуется](https://github.com/OpenByteDev/wineventhook-rs/issues/1).

##### Примеры кода (`win_event_hook`)

```rust
// Cargo.toml
// [dependencies]
// win_event_hook = "0.4"
// windows = { version = "0.58", features = [
//     "Win32_UI_WindowsAndMessaging",
//     "Win32_System_Threading",
//     "Win32_Foundation"
// ]}

use win_event_hook::events::{Event, NamedEvent};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let blocked_apps = vec![
        "chrome.exe",
        "discord.exe",
        "telegram.exe",
    ];

    // Подписываемся на события появления окон на переднем плане
    let config = win_event_hook::Config::builder()
        .skip_own_process()          // игнорировать свои окна
        .with_dedicated_thread()     // хук на отдельном потоке
        .with_events(vec![
            Event::Named(NamedEvent::SystemForeground),  // окно вышло на передний план
            Event::Named(NamedEvent::ObjectShow),        // объект стал видимым
            Event::Named(NamedEvent::ObjectCreate),      // создан UI-элемент
        ])
        .finish();

    let handler = move |_event, _hwnd, _id_object, _id_child, _event_thread, _event_time| {
        // Здесь: получить HWND → PID → путь к exe → проверить → закрыть/убить
        // (см. логику ниже)
    };

    let hook = win_event_hook::WinEventHook::install(config, handler)?;

    // Главный цикл приложения...
    // hook автоматически снимется при drop

    Ok(())
}
```

Получение PID по HWND:

```rust
use windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION,
    PROCESS_TERMINATE, TerminateProcess
};
use windows::Win32::Foundation::CloseHandle;

unsafe fn get_exe_path_for_window(hwnd: windows::Win32::Foundation::HWND) -> Option<String> {
    let mut pid: u32 = 0;
    GetWindowThreadProcessId(hwnd, Some(&mut pid));
    if pid == 0 { return None; }

    let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;

    let mut buf = [0u16; 1024];
    let mut size = buf.len() as u32;
    QueryFullProcessImageNameW(handle, Default::default(),
                                windows::core::PWSTR(buf.as_mut_ptr()), &mut size).ok()?;
    let _ = CloseHandle(handle);

    Some(String::from_utf16_lossy(&buf[..size as usize]))
}

unsafe fn kill_process(pid: u32) -> bool {
    if let Ok(handle) = OpenProcess(PROCESS_TERMINATE, false, pid) {
        let result = TerminateProcess(handle, 1);
        let _ = CloseHandle(handle);
        result.is_ok()
    } else {
        false
    }
}
```

#### Достоинства

- Быстрая реакция
- Не требуются права администратора
- При завершении процесса хук снимается

#### Недостатки

- Процессы без окон не отлавливаются

### Слой 2: Опрашивание процессов

#### Описание

Каждые 200–500 мс перечислять все запущенные процессы и убивать заблокированные.
Это страховка для случаев, которые не поймал слой 1 (процессы без окон,
промежуточные состояния).

#### Детали реализации

Крейт `sysinfo` предоставляет удобную обёртку для получения информации о
запущенных процессах и является кроссплатформенным.

Следует использовать функцию
[`refresh_processes_specifics`](https://docs.rs/sysinfo/latest/sysinfo/struct.System.html#method.refresh_processes_specifics)
с `ProcessRefreshKind::nothing().without_tasks().with_cmd()`.

`without_tasks()` необходимо, потому что даже `RefreshKind::nothing()` не
отменяет сбор информации о задачах. Сбор информации без задач значительно дешевле.

Также рекомендуется создавать экземпляр `System` один раз и переиспользовать
его, потому что многие метрики (например, использование ЦП) вычисляются как разница с
предыдущим измерением, а первоначальное выделение памяти под список процессов
занимает заметное время.

##### Примеры кода

```rust
// Cargo.toml
// [dependencies]
// sysinfo = "0.38"

use sysinfo::{System, ProcessesToUpdate, ProcessRefreshKind};
use std::collections::HashSet;
use std::path::Path;

struct ProcessBlocker {
    sys: System,
    blocked: HashSet<String>,  // например: {"telegram.exe", "discord.exe"}
}

impl ProcessBlocker {
    fn new(blocked_apps: Vec<String>) -> Self {
        Self {
            // Обновляем только список процессов с минимумом дополнительной
            // информации
            sys: System::new_with_specifics(
                RefreshKind::nothing()
                    .with_processes(
                        ProcessRefreshKind::nothing()
                            .without_tasks()
                            .with_cmd()
                    )
            ),
            blocked: blocked_apps.into_iter().collect(),
        }
    }

    fn scan_and_kill(&mut self) {
        self.sys.refresh_processes(ProcessesToUpdate::All, true);

        for (pid, process) in self.sys.processes() {
            let exe_name = process.name().to_string_lossy().to_lowercase();

            if self.blocked.contains(&exe_name) {
                println!("Блокирую: {} (PID {})", exe_name, pid);
                process.kill();
            }
        }
    }
}

// В основном цикле:
fn blocking_loop(mut blocker: ProcessBlocker) {
    loop {
        blocker.scan_and_kill();
        std::thread::sleep(std::time::Duration::from_millis(300));
    }
}
```

#### Достоинства

- Относительно быстрая реакция (200-500 мс)
- Права администратора не требуются
- Fail-safe (если приложение не работает, процессы не убиваются)
- Приложение будет остановлено, даже если оно не имеет окна

#### Недостатки

- Задержка реакции присутствует - окно мелькает
- Обходится переименованием исполняемого файла (сложно решить, поскольку путь - основной метод идентификации исполняемого файла)

### Замечания о деструктивности

Никакие приложения не блокируются, если приложение не запущено, поэтому
аварийное завершение не повлечет нарушений в работе системы.

Подписка на оконные события автоматически снимается при завершении работы
приложения.

## Linux

### Слой 1: Polkit-помощник для bind-mount

Слой подразумевает создание специального демона с повышенными правами, в
обязанности которого входит:

- Монтирование `/dev/null` на место исполняемых файлов отвлекающих приложений во
  время периода концентрации;
- Размонтирование `/dev/null` с места исполняемых файлов отвлекающих приложений
  после окончания периода концентрации (в том числе при аварийном завершении
  Quilum/демона);
- Отслеживание изменений путей до исполняемых файлов отвлекающих приложений с
  помощью `inotify` (см. крейт [inotify](https://docs.rs/inotify));
- Добавление обновленных путей в базу данных при обнаружении изменений и
  уведомление Quilum об изменениях, чтобы приложение внесло изменения в базу

Любые другие действия (в частности, самостоятельное внесение изменений в список
отвлекающих приложений) демон совершать не вправе из соображений безопасности.

Коммуникация между демоном и приложением осуществляется через D-Bus:

- Когда начинается очередной период концентрации:
  - Quilum отправляет сигнал демону
  - Демон:
    - собирает список исполняемых файлов отвлекающих приложений из общей базы
      данных;
    - делает bind-mount `/dev/null` для каждого из исполняемых файлов
- Когда заканчивается очередной период концентрации:
  - Quilum отправляет сигнал демону
  - Демон:
    - удаляет bind-mount `/dev/null` для каждого из исполняемых файлов

**bind mount** - эквивалент вызова `sudo mount --bind /dev/null /path/to/app`,
где `/path/to/app` - путь до приложения.

Реализацию монтирования можно реализовать с помощью крейта
[nix](https://docs.rs/nix) (см. модуль
[`nix::mount`](https://docs.rs/nix/latest/nix/mount/index.html)).

Метод с удалением прав на исполнение (рассматривался ранее при разработке) не
используется, поскольку не применим к файловым системам, примонтированным только
для чтения (там права менять вообще нельзя).

### Слой 2: Опрашивание процессов

Аналогично опрашиванию процессов на Windows.

### Достоинства

- Первый слой полностью превентивный - заблокированное приложение почти
  невозможно запустить (см. недостатки).
- Последствия сбоев легко исправляются (см. замечания о деструктивности)
- Сам файл остается нетронутым. Даже его метаданные не изменяются.
- Даже если приложение удалось запустить через жесткую ссылку, оно будет
  заблокировано вторым слоем.

### Недостатки

- Если пользователь создаст жесткую ссылку на заблокированный файл, то он сможет
  запустить его через эту ссылку. Поскольку inode файла не хранит список жестких
  ссылок, тривиального способа найти все жесткие ссылки нет.

### Замечания о деструктивности

Монтирование поверх файла деструктивно в том смысле, что без активных действий
блокировку не снять. Поэтому при аварийном завершении Quilum или
вспомогательного демона приложению или, в крайнем случае, пользователю
придется предпринять определенные действия для снятия ограничений.

Для решения этой проблемы будем требовать от Quilum сообщать время окончания
сессии концентрации. Systemd будет перезапускать демон, если он будет аварийно
завершаться. Демон должен будет проверять при очередном запуске время окончания
последней сессии концентрации и, если оно истекло, снимать блокировку.

Для разблокировки достаточно выполнить одно из двух:

- Явно размонтировать `/dev/null` для каждого из исполняемых файлов
- Перезагрузить компьютер (перелогин, по словам Opus, не поможет)

## macOS

### Слой 1: NSWorkspace Notifications

Метод аналогичен `SetWinEventHook` на Windows.

macOS предоставляет уведомление `didLaunchApplicationNotification`, которое
рабочее пространство публикует при запуске нового приложения.

```
Пользователь запускает Firefox
    │
    ▼
macOS WindowServer / LaunchServices
    │
    ├── NSWorkspace публикует didLaunchApplicationNotification
    │       │
    │       ▼
    │   Quilum получает уведомление:
    │   {
    │     bundleIdentifier: "org.mozilla.firefox",
    │     processIdentifier: 12345,
    │     localizedName: "Firefox",
    │     bundleURL: "/Applications/Firefox.app"
    │   }
    │       │
    │       ▼
    │   В блоклисте? → Если да, то terminate() или forceTerminate()
    │
    └── Firefox появляется на экране на долю секунды, затем закрывается
```

Примерная реализация на Rust (составлена Opus):

```rust
// Cargo.toml:
// [dependencies]
// objc2 = "0.6"
// objc2-foundation = { version = "0.3", features = ["NSNotification", "NSString", 
//                      "NSDictionary", "NSRunLoop", "NSDate"] }
// objc2-app-kit = { version = "0.3", features = ["NSWorkspace", 
//                   "NSRunningApplication"] }

use objc2_foundation::{NSNotification, NSNotificationCenter, NSString, 
                        NSRunLoop, NSDate, NSDefaultRunLoopMode};
use objc2_app_kit::{NSWorkspace, NSRunningApplication};
use objc2::runtime::ProtocolObject;
use objc2::{declare_class, msg_send_id, mutability, ClassType, DeclaredClass};
use std::collections::HashSet;
use std::sync::Mutex;

// Список заблокированных bundle identifiers
static BLOCKED_BUNDLES: Mutex<HashSet<String>> = Mutex::new(HashSet::new());

declare_class!(
    struct AppMonitor;
    
    unsafe impl ClassType for AppMonitor {
        type Super = objc2_foundation::NSObject;
        type Mutability = mutability::Mutable;
        const NAME: &'static str = "AppMonitor";
    }

    impl DeclaredClass for AppMonitor {
        type Ivars = ();
    }

    // Callback при запуске приложения
    unsafe impl AppMonitor {
        #[method(appDidLaunch:)]
        unsafe fn app_did_launch(&self, notification: &NSNotification) {
            let user_info = notification.userInfo().unwrap();
            let key = NSString::from_str("NSWorkspaceApplicationKey");
            
            if let Some(app) = user_info.objectForKey(&key) {
                // Приведение к NSRunningApplication
                let app: &NSRunningApplication = unsafe { &*(app as *const _ as *const _) };

                if let Some(bundle_id) = app.bundleIdentifier() {
                    let bundle_str = bundle_id.to_string();
                    
                    let blocked = BLOCKED_BUNDLES.lock().unwrap();
                    if blocked.contains(&bundle_str) {
                        println!("⛔ Blocking: {} ({})", 
                                 app.localizedName().unwrap_or_default(), 
                                 bundle_str);
                        
                        // Вежливая попытка закрыть
                        let terminated = app.terminate();
                        
                        if !terminated {
                            // Принудительное завершение (аналог SIGKILL)
                            app.forceTerminate();
                        }
                    }
                }
            }
        }
    }
);

fn start_monitoring() {
    // Заполняем блоклист
    {
        let mut blocked = BLOCKED_BUNDLES.lock().unwrap();
        blocked.insert("com.discord.Discord".to_string());
        blocked.insert("ru.keepcoder.Telegram".to_string());
    }
    
    unsafe {
        let workspace = NSWorkspace::sharedWorkspace();
        let notification_center = workspace.notificationCenter();
        let monitor = AppMonitor::new();
        
        // Подписка на уведомления о запуске приложений
        notification_center.addObserver_selector_name_object(
            &monitor,
            objc2::sel!(appDidLaunch:),
            Some(&NSString::from_str("NSWorkspaceDidLaunchApplicationNotification")),
            None,
        );
        
        // Также подписываемся на активацию (переключение на приложение)
        notification_center.addObserver_selector_name_object(
            &monitor,
            objc2::sel!(appDidLaunch:),  // тот же handler
            Some(&NSString::from_str("NSWorkspaceDidActivateApplicationNotification")),
            None,
        );
        
        // Запускаем run loop (необходим для получения уведомлений)
        let run_loop = NSRunLoop::currentRunLoop();
        loop {
            run_loop.runMode_beforeDate(
                NSDefaultRunLoopMode,
                &NSDate::distantFuture(),
            );
        }
    }
}
```

- Привилегии не требуются
- Работает для всех графических приложений
- Не подразумевает опрашивание
- Идентификация по App Bundle ID
- Полностью fail-safe по аналогичным причинам

Подход требует версию macOS не ниже 10.12 (требовние крейта `objc2`).

Подробнее - см. [поддерживаемые крейтом версии ОС](https://docs.rs/objc2/latest/objc2/#supported-operating-systems).

## Источники

- Общие
  - Anthropic Claude Opus 4.6 (использовался через [Arena](https://arena.ai))
- Windows
  - https://docs.rs/win_event_hook
  - https://docs.rs/wineventhook
  - _[Не для презентации]_
    https://github.com/OpenByteDev/wineventhook-rs/issues/1
  - https://docs.rs/windows
  - https://docs.rs/sysinfo
  - https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwineventhook
- Linux
  - https://www.freedesktop.org/software/polkit/docs/latest/polkit.8.html
  - https://man7.org/linux/man-pages/man7/inotify.7.html
  - https://docs.rs/inotify
  - https://docs.rs/nix
- macOS
  - https://developer.apple.com/documentation/appkit/nsworkspace/didlaunchapplicationnotification
