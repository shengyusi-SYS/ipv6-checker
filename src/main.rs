mod config;
mod handlers;
mod service;

use std::env;
use std::ffi::OsString;
use std::sync::mpsc;
use std::time::Duration;
use windows_service::{
    define_windows_service,
    service::{
        ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
        ServiceType,
    },
    service_control_handler::{self, ServiceControlHandlerResult},
    service_dispatcher,
};

const SERVICE_NAME: &str = "IPv6Checker";
const SERVICE_TYPE: ServiceType = ServiceType::OWN_PROCESS;

define_windows_service!(ffi_service_main, windows_service_main);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    // 判断运行模式
    if args.len() > 1 {
        match args[1].as_str() {
            "install" => return install_service(),
            "uninstall" => return uninstall_service(),
            "start" => return start_service(),
            "stop" => return stop_service(),
            "restart" => return restart_service(),
            "status" => return show_status(),
            "help" | "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            _ => {
                println!("Unknown command: {}", args[1]);
                println!("Use 'help' to see available commands");
                return Ok(());
            }
        }
    }

    // 尝试作为Windows服务运行
    if let Err(_e) = service_dispatcher::start(SERVICE_NAME, ffi_service_main) {
        // 如果不是服务模式，则作为控制台应用运行
        run_console_mode()?;
    }

    Ok(())
}

fn print_help() {
    println!("IPv6 Checker Service - 命令行工具");
    println!();
    println!("用法: ipv6-checker.exe [COMMAND]");
    println!();
    println!("命令:");
    println!("  install      安装Windows服务（需要管理员权限）");
    println!("  uninstall    卸载Windows服务（需要管理员权限）");
    println!("  start        启动服务（需要管理员权限）");
    println!("  stop         停止服务（需要管理员权限）");
    println!("  restart      重启服务（需要管理员权限）");
    println!("  status       查看服务状态");
    println!("  help         显示此帮助信息");
    println!();
    println!("如果不带任何参数运行，程序将:");
    println!("  - 尝试作为Windows服务运行（如果已安装）");
    println!("  - 或作为控制台应用运行（用于测试）");
    println!();
    println!("示例:");
    println!("  ipv6-checker.exe install    # 安装服务");
    println!("  ipv6-checker.exe start      # 启动服务");
    println!("  ipv6-checker.exe status     # 查看状态");
    println!("  ipv6-checker.exe            # 控制台模式运行");
}

fn windows_service_main(_arguments: Vec<OsString>) {
    if let Err(e) = run_service() {
        eprintln!("Service error: {}", e);
    }
}

fn run_service() -> Result<(), Box<dyn std::error::Error>> {
    let (shutdown_tx, shutdown_rx) = mpsc::channel();

    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Stop | ServiceControl::Interrogate => {
                shutdown_tx.send(()).ok();
                ServiceControlHandlerResult::NoError
            }
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)?;

    // 通知服务控制管理器服务正在启动
    status_handle.set_service_status(ServiceStatus {
        service_type: SERVICE_TYPE,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    // 在新线程中运行服务器
    let _server_handle = std::thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            if let Err(e) = service::run_server().await {
                eprintln!("Server error: {}", e);
            }
        });
    });

    // 等待停止信号
    shutdown_rx.recv().ok();

    // 通知服务控制管理器服务正在停止
    status_handle.set_service_status(ServiceStatus {
        service_type: SERVICE_TYPE,
        current_state: ServiceState::Stopped,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    Ok(())
}

fn run_console_mode() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    tracing::info!("Starting as console application");

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async { service::run_server().await })?;

    Ok(())
}

fn install_service() -> Result<(), Box<dyn std::error::Error>> {
    use windows_service::service::{
        ServiceAccess, ServiceErrorControl, ServiceInfo, ServiceStartType,
    };
    use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};

    let manager =
        ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CREATE_SERVICE)?;

    let exe_path = std::env::current_exe()?;

    let service_info = ServiceInfo {
        name: OsString::from(SERVICE_NAME),
        display_name: OsString::from("IPv6 Checker Service"),
        service_type: ServiceType::OWN_PROCESS,
        start_type: ServiceStartType::AutoStart,
        error_control: ServiceErrorControl::Normal,
        executable_path: exe_path,
        launch_arguments: vec![],
        dependencies: vec![],
        account_name: None,
        account_password: None,
    };

    let service = manager.create_service(&service_info, ServiceAccess::CHANGE_CONFIG)?;
    service.set_description("IPv6 address checker service")?;

    println!("✓ 服务 '{}' 安装成功!", SERVICE_NAME);
    println!();
    println!("服务配置:");
    println!("  显示名称: IPv6 Checker Service");
    println!("  启动类型: 自动（随系统启动）");
    println!("  运行账户: LocalSystem（系统账户）");
    println!("  说明: 可在用户登录前运行");
    println!();
    println!("下一步:");
    println!("  1. 确保 config.json 在可执行文件同目录");
    println!("  2. 使用 'ipv6-checker.exe start' 启动服务");
    println!("  或重启计算机后服务将自动启动");

    Ok(())
}

fn uninstall_service() -> Result<(), Box<dyn std::error::Error>> {
    use windows_service::service::ServiceAccess;
    use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};

    let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)?;
    let service = manager.open_service(SERVICE_NAME, ServiceAccess::DELETE)?;
    service.delete()?;

    println!("✓ 服务 '{}' 卸载成功!", SERVICE_NAME);

    Ok(())
}

fn start_service() -> Result<(), Box<dyn std::error::Error>> {
    use windows_service::service::ServiceAccess;
    use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};

    println!("正在启动服务 '{}'...", SERVICE_NAME);

    let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)?;
    let service = manager.open_service(SERVICE_NAME, ServiceAccess::START)?;
    service.start(&[] as &[&str])?;

    println!("✓ 服务启动成功!");

    Ok(())
}

fn stop_service() -> Result<(), Box<dyn std::error::Error>> {
    use windows_service::service::ServiceAccess;
    use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};

    println!("正在停止服务 '{}'...", SERVICE_NAME);

    let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)?;
    let service = manager.open_service(SERVICE_NAME, ServiceAccess::STOP)?;
    service.stop()?;

    println!("✓ 服务停止成功!");

    Ok(())
}

fn restart_service() -> Result<(), Box<dyn std::error::Error>> {
    println!("正在重启服务 '{}'...", SERVICE_NAME);

    if let Err(e) = stop_service() {
        println!("警告: 停止服务时出错: {}", e);
    }

    // 等待服务完全停止
    std::thread::sleep(Duration::from_secs(2));

    start_service()?;

    Ok(())
}

fn show_status() -> Result<(), Box<dyn std::error::Error>> {
    use windows_service::service::ServiceAccess;
    use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};

    let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)?;

    match manager.open_service(SERVICE_NAME, ServiceAccess::QUERY_STATUS) {
        Ok(service) => {
            let status = service.query_status()?;

            println!("服务名称: {}", SERVICE_NAME);
            println!("显示名称: IPv6 Checker Service");
            print!("当前状态: ");

            match status.current_state {
                ServiceState::Stopped => println!("已停止"),
                ServiceState::StartPending => println!("正在启动..."),
                ServiceState::StopPending => println!("正在停止..."),
                ServiceState::Running => println!("运行中 ✓"),
                ServiceState::ContinuePending => println!("正在继续..."),
                ServiceState::PausePending => println!("正在暂停..."),
                ServiceState::Paused => println!("已暂停"),
            }

            println!("进程ID: {:?}", status.process_id);
        }
        Err(_) => {
            println!("服务 '{}' 未安装", SERVICE_NAME);
            println!("使用 'ipv6-checker.exe install' 来安装服务");
        }
    }

    Ok(())
}
