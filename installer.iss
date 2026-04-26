[Setup]
AppName=APPSearch
AppVersion=0.1.0
AppPublisher=SimulQuest
AppPublisherURL=https://appsearch.rf.gd
AppSupportURL=https://github.com/simulquest/appsearch/issues
AppUpdatesURL=https://github.com/simulquest/appsearch/releases
DefaultDirName={autopf}\APPSearch
DefaultGroupName=APPSearch
OutputBaseFilename=APPSearch_Setup
Compression=lzma
SolidCompression=yes
PrivilegesRequired=admin
ArchitecturesAllowed=x64
ArchitecturesInstallIn64BitMode=x64
AppMutex=Global\APPSearch_Mutex
UninstallDisplayIcon={app}\appsearch.exe
SetupIconFile=assets\logo.ico
CloseApplications=yes

[Files]
Source: "target\release\appsearch.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\APPSearch"; Filename: "{app}\appsearch.exe"
Name: "{autodesktop}\APPSearch"; Filename: "{app}\appsearch.exe"; Tasks: desktopicon
Name: "{userstartup}\APPSearch"; Filename: "{app}\appsearch.exe"; Check: ShouldInstallStartup

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[UninstallDelete]
Type: files; Name: "{app}\shortcut.txt"

[Code]
var
  ShortcutPage: TInputQueryWizardPage;
  StartupPage: TInputOptionWizardPage;  

procedure InitializeWizard;
begin
  // Page 1: Raccourci
  ShortcutPage := CreateInputQueryPage(wpWelcome, 'Configuration', 'Paramètres requis', 'Veuillez configurer votre raccourci clavier préféré.');
  ShortcutPage.Add('Raccourci clavier (ex: Ctrl+Shift+K):', False);
  ShortcutPage.Values[0] := 'Ctrl+Shift+K';

  // Page 2: Démarrage (via TInputOptionWizardPage)
  StartupPage := CreateInputOptionPage(ShortcutPage.ID, 'Démarrage', 'Options', 'Voulez-vous lancer APPSearch au démarrage de Windows ?', False, True);
  StartupPage.Add('Lancer APPSearch au démarrage de Windows');
  StartupPage.Values[0] := True;
end;

function ShouldInstallStartup: Boolean;
begin
  Result := StartupPage.Values[0];
end;

procedure CurStepChanged(CurStep: TSetupStep);
var
  ShortcutValue: string;
begin
  if CurStep = ssPostInstall then
  begin
    // Sauvegarder le raccourci
    ShortcutValue := ShortcutPage.Values[0];
    SaveStringToFile(ExpandConstant('{app}\shortcut.txt'), ShortcutValue, False);
  end;
end;

[Run]
Filename: "{app}\appsearch.exe"; Description: "Lancer APPSearch"; Flags: nowait postinstall skipifsilent