const vscode = require('vscode');

/**
 * @param {vscode.ExtensionContext} context
 */
function activate(context) {
  console.log('⚡ Titanium (Ti22) Language Extension is now active!');

  // 1. Status Bar Item
  const statusBar = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
  statusBar.text = '$(zap) Titanium';
  statusBar.tooltip = 'Titanium Native Web Runtime (Ti22)';
  statusBar.command = 'titanium.showMenu';
  statusBar.show();
  context.subscriptions.push(statusBar);

  // 2. Command Palette Menu
  const menuCmd = vscode.commands.registerCommand('titanium.showMenu', async () => {
    const choice = await vscode.window.showQuickPick([
      { label: '$(play) Start Dev Server', description: 'Run titanium dev with soft-DOM live reload', cmd: 'titanium.startDev' },
      { label: '$(browser) Open Web Studio', description: 'Open visual database & query studio (/__titanium_studio)', cmd: 'titanium.openStudio' },
      { label: '$(database) Run Migrations', description: 'Execute pending SQL migrations in migrations/', cmd: 'titanium.runMigrations' },
      { label: '$(package) Build Package', description: 'Validate and package project for production', cmd: 'titanium.build' },
      { label: '$(info) Project Diagnostics', description: 'Display runtime telemetry and routes', cmd: 'titanium.info' }
    ], { placeHolder: 'Select a Titanium Action' });

    if (choice) {
      vscode.commands.executeCommand(choice.cmd);
    }
  });
  context.subscriptions.push(menuCmd);

  // 3. Start Dev Server Command
  const startDevCmd = vscode.commands.registerCommand('titanium.startDev', () => {
    const terminal = getOrCreateTerminal();
    terminal.show();
    terminal.sendText('titanium dev .');
    vscode.window.showInformationMessage('⚡ Titanium dev server started on http://127.0.0.1:8080');
  });
  context.subscriptions.push(startDevCmd);

  // 4. Open Studio Command
  const openStudioCmd = vscode.commands.registerCommand('titanium.openStudio', () => {
    vscode.env.openExternal(vscode.Uri.parse('http://127.0.0.1:8080/__titanium_studio'));
  });
  context.subscriptions.push(openStudioCmd);

  // 5. Run Migrations Command
  const migrateCmd = vscode.commands.registerCommand('titanium.runMigrations', () => {
    const terminal = getOrCreateTerminal();
    terminal.show();
    terminal.sendText('titanium migrate .');
  });
  context.subscriptions.push(migrateCmd);

  // 6. Build Command
  const buildCmd = vscode.commands.registerCommand('titanium.build', () => {
    const terminal = getOrCreateTerminal();
    terminal.show();
    terminal.sendText('titanium build .');
  });
  context.subscriptions.push(buildCmd);

  // 7. Info Command
  const infoCmd = vscode.commands.registerCommand('titanium.info', () => {
    const terminal = getOrCreateTerminal();
    terminal.show();
    terminal.sendText('titanium info .');
  });
  context.subscriptions.push(infoCmd);
}

function getOrCreateTerminal() {
  const existing = vscode.window.terminals.find(t => t.name === 'Titanium Runtime');
  if (existing) return existing;
  return vscode.window.createTerminal('Titanium Runtime');
}

function deactivate() {}

module.exports = {
  activate,
  deactivate
};
