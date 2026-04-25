[Setup]
AppId={{9B8F421C-5C9B-4D1E-B234-7D688CD4B9AB}
AppName=Focus Timer
AppVersion={#MyAppVersion}
AppPublisher=aianau
DefaultDirName={autopf}\FocusTimerRust
DefaultGroupName=Focus Timer
OutputBaseFilename=FocusTimerRust-Setup
Compression=lzma
SolidCompression=yes
WizardStyle=modern
PrivilegesRequired=admin

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "target\release\focus-timer-rust.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "assets\*"; DestDir: "{commonappdata}\FocusTimerRust\assets"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{group}\Focus Timer"; Filename: "{app}\focus-timer-rust.exe"
Name: "{autodesktop}\Focus Timer"; Filename: "{app}\focus-timer-rust.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\focus-timer-rust.exe"; Description: "{cm:LaunchProgram,Focus Timer}"; Flags: nowait postinstall skipifsilent
