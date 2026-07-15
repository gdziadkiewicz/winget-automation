#define AppName "WinGet Update Monitor"
#ifndef AppVersion
#define AppVersion "0.1.0"
#endif
#define AppExeName "winget-automation.exe"
#define AppUrl "https://github.com/gdziadkiewicz/winget-automation"

[Setup]
AppId={{A558DA5B-2A50-45D9-98F4-A5BE56883776}
AppName={#AppName}
AppVersion={#AppVersion}
AppPublisher=winget-automation contributors
AppPublisherURL={#AppUrl}
AppSupportURL={#AppUrl}/issues
AppUpdatesURL={#AppUrl}/releases
DefaultDirName={localappdata}\Programs\{#AppName}
DefaultGroupName={#AppName}
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
UsedUserAreasWarning=no
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0.17763
OutputDir=..\target\installer
OutputBaseFilename=WinGet-Update-Monitor-{#AppVersion}-Setup
SetupIconFile=..\assets\tray-icon.ico
UninstallDisplayIcon={app}\tray-icon.ico
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
CloseApplications=yes
CloseApplicationsFilter={#AppExeName}
RestartApplications=no
VersionInfoVersion={#AppVersion}
VersionInfoDescription={#AppName} installer
VersionInfoProductName={#AppName}
VersionInfoProductVersion={#AppVersion}

[Files]
Source: "..\target\x86_64-pc-windows-msvc\release\{#AppExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\assets\tray-icon.ico"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#AppName}"; Filename: "{app}\{#AppExeName}"; IconFilename: "{app}\tray-icon.ico"; AppUserModelID: "gdziadkiewicz.WinGetUpdateMonitor"

[Registry]
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "winget-automation"; ValueData: """{app}\{#AppExeName}"""; Flags: uninsdeletevalue

[Run]
Filename: "{app}\{#AppExeName}"; Description: "Launch {#AppName}"; Flags: nowait postinstall skipifsilent

[UninstallRun]
Filename: "{sys}\taskkill.exe"; Parameters: "/F /IM {#AppExeName}"; Flags: runhidden waituntilterminated; RunOnceId: "StopApp"
