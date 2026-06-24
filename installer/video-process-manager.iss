#define MyAppName "视频应用进程管理工具"
#define MyAppVersion "0.1.0"
#define MyAppPublisher "Dawning02"
#define MyAppExeName "video-process-manager.exe"

[Setup]
AppId={{F38D5E66-3456-4B37-8B31-F0B19D58D2E5}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
DefaultDirName={localappdata}\VideoProcessManager
DefaultGroupName={#MyAppName}
DisableProgramGroupPage=yes
OutputDir=..\dist
OutputBaseFilename=VideoProcessManagerSetup-{#MyAppVersion}
Compression=lzma
SolidCompression=yes
WizardStyle=modern
PrivilegesRequired=lowest
UninstallDisplayIcon={app}\{#MyAppExeName}

[Languages]
Name: "chinesesimp"; MessagesFile: "compiler:Languages\ChineseSimplified.isl"

[Tasks]
Name: "desktopicon"; Description: "创建桌面快捷方式"; GroupDescription: "附加快捷方式："; Flags: unchecked

[Files]
Source: "..\dist\VideoProcessManager\video-process-manager.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\dist\VideoProcessManager\presets.toml"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\dist\VideoProcessManager\config.toml"; DestDir: "{app}"; Flags: onlyifdoesntexist

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; WorkingDir: "{app}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; WorkingDir: "{app}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "启动 {#MyAppName}"; Flags: nowait postinstall skipifsilent; WorkingDir: "{app}"
