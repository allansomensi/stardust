; Stardust installer (Inno Setup 6 — https://jrsoftware.org/isinfo.php)
;
; How to build:
;   1. On Windows, build the game:   cargo build --release
;   2. Open this file in Inno Setup and click "Compile" (or run: iscc installer\stardust.iss)
;   3. The installer is written to dist\Stardust-0.1.0-setup.exe
;
; If you built the game another way (e.g. cross-compiling), adjust ExePath below.

#define AppName "Stardust"
#define AppVersion "0.1.0"
#define AppPublisher "Allan Somensi"
#ifndef ExePath
  #define ExePath "..\target\release\stardust.exe"
#endif

[Setup]
AppId={{3FFB74CC-7461-46AE-8603-26436A9CA035}
AppName={#AppName}
AppVersion={#AppVersion}
AppVerName={#AppName} {#AppVersion}
AppPublisher={#AppPublisher}
DefaultDirName={autopf}\{#AppName}
DefaultGroupName={#AppName}
DisableProgramGroupPage=yes
OutputDir=..\dist
OutputBaseFilename=Stardust-{#AppVersion}-setup
SetupIconFile=..\assets\stardust.ico
UninstallDisplayIcon={app}\stardust.exe
UninstallDisplayName={#AppName}
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
PrivilegesRequiredOverridesAllowed=dialog
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "brazilianportuguese"; MessagesFile: "compiler:Languages\BrazilianPortuguese.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"

[Files]
Source: "{#ExePath}"; DestDir: "{app}"; DestName: "stardust.exe"; Flags: ignoreversion
Source: "..\README.txt"; DestDir: "{app}"; Flags: ignoreversion isreadme; Languages: english
Source: "..\README.txt"; DestDir: "{app}"; Flags: ignoreversion; Languages: brazilianportuguese
Source: "..\LEIAME.txt"; DestDir: "{app}"; Flags: ignoreversion isreadme; Languages: brazilianportuguese
Source: "..\LEIAME.txt"; DestDir: "{app}"; Flags: ignoreversion; Languages: english

[Icons]
Name: "{group}\{#AppName}"; Filename: "{app}\stardust.exe"; WorkingDir: "{app}"
Name: "{group}\Readme"; Filename: "{app}\README.txt"
Name: "{group}\Leia-me"; Filename: "{app}\LEIAME.txt"
Name: "{group}\{cm:UninstallProgram,{#AppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\stardust.exe"; WorkingDir: "{app}"; Tasks: desktopicon

[Run]
Filename: "{app}\stardust.exe"; Description: "{cm:LaunchProgram,{#AppName}}"; Flags: nowait postinstall skipifsilent
