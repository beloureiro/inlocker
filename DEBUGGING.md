# InLocker - Debugging Guide

## 📊 Como Ver Logs e Status dos Backups

### 1. **Logs do Frontend (Browser Console)**

**Como abrir:**
- **macOS:** `Cmd + Option + I` → Aba "Console"
- **Menu:** View → Developer → Toggle Developer Tools

**O que você verá:**
```
[BackupList] Starting backup for config: backup-1234567890
[BackupList] Calling run_backup_now...
[BackupList] Backup result: { success: true, message: "..." }
[BackupList] Backup successful!
[BackupList] Backup process finished
```

**Erros aparecem em vermelho:**
```
[BackupList] Backup error: Failed to read source folder
```

---

### 2. **Logs do Backend (Terminal)** ✅ SEMPRE HABILITADO

**Onde ver:**
- No terminal onde você executou `pnpm tauri dev`

**Logs são habilitados automaticamente!**
```bash
# Basta rodar normalmente - logs já estão habilitados:
pnpm tauri dev
```

**O que você verá durante um backup:**
```
[2025-11-01T14:14:48Z INFO  inlocker_lib] InLocker starting...
[2025-11-01T14:15:20Z INFO  inlocker_lib] 🔵 Starting INCREMENTAL backup
[2025-11-01T14:15:20Z INFO  inlocker_lib] 📂 Source: /Users/blc/Dev
[2025-11-01T14:15:20Z INFO  inlocker_lib] 💾 Destination: /Users/blc/Documents/Dev-Bkp
[2025-11-01T14:15:20Z INFO  inlocker_lib] 📋 Scanning files...
[2025-11-01T14:15:21Z INFO  inlocker_lib] ✅ Found 123 files (4.50 MB)
[2025-11-01T14:15:21Z INFO  inlocker_lib] 📦 Creating TAR archive...
[2025-11-01T14:15:22Z INFO  inlocker_lib] ✅ TAR archive created (4.50 MB)
[2025-11-01T14:15:22Z INFO  inlocker_lib] 🗜️  Compressing with zstd (level 3)...
[2025-11-01T14:15:23Z INFO  inlocker_lib] ✅ Compressed to 1.20 MB (73.3% compression)
[2025-11-01T14:15:23Z INFO  inlocker_lib] 💾 Writing backup file: backup_incr_20251101_143052.tar.zst
[2025-11-01T14:15:23Z INFO  inlocker_lib] ✅ Backup file saved
[2025-11-01T14:15:23Z INFO  inlocker_lib] 🔒 Calculating SHA-256 checksum...
[2025-11-01T14:15:23Z INFO  inlocker_lib] ✅ Checksum: 3a7f2c1d8e9b...
[2025-11-01T14:15:23Z INFO  inlocker_lib] 🎉 Backup completed successfully in 3s
```

**Erros aparecem como:**
```
[2025-11-01T14:16:05Z ERROR inlocker_lib] Backup failed: No such file or directory
```

---

### 3. **Feedback Visual na UI**

#### **Durante o Backup:**
- ✅ **Spinner animado** azul
- ✅ Mensagem: "Backup in progress..."
- ✅ Botão "Run Backup" desabilitado

#### **Após Sucesso:**
- ✅ **Caixa verde** com ícone de checkmark
- ✅ Título: "Backup Successful"
- ✅ Detalhes: "123 files, 4.5 MB → 1.2 MB (73.3% compression)"

#### **Após Erro:**
- ✅ **Caixa vermelha** com ícone de X
- ✅ Título: "Backup Failed"
- ✅ Mensagem de erro detalhada
- ✅ **Alert popup** com erro completo

---

### 4. **Logs de Backups Agendados (launchd)**

**Localização dos logs:**
```bash
# Logs de stdout
tail -f /tmp/inlocker-backup-<config_id>.log

# Logs de erros
tail -f /tmp/inlocker-backup-<config_id>.err
```

**Exemplo:**
```bash
# Ver logs do último backup
tail -20 /tmp/inlocker-backup-*.log
```

---

## 🔍 Troubleshooting

### Backup não inicia

**1. Verificar no console do browser:**
```
[BackupList] Backup error: Config not found
```

**2. Verificar permissões de pastas:**
```bash
# Testar leitura da pasta fonte
ls -la /path/to/source/folder

# Testar escrita na pasta destino
touch /path/to/destination/test.txt
rm /path/to/destination/test.txt
```

**3. Verificar se a pasta existe:**
- Fonte: Deve existir e ter arquivos
- Destino: Deve existir e ter permissão de escrita

---

### Backup falha com "Permission Denied"

**Solução:**
1. Abrir: **System Settings** → **Privacy & Security** → **Files and Folders**
2. Encontrar **InLocker**
3. Habilitar acesso às pastas necessárias

---

### Schedule não está funcionando

**1. Verificar se .plist foi criado:**
```bash
ls -la ~/Library/LaunchAgents/com.inlocker.backup.*
```

**2. Verificar se está carregado:**
```bash
launchctl list | grep inlocker
```

**3. Ver detalhes do agendamento:**
```bash
launchctl print gui/$(id -u)/com.inlocker.backup.backup-123
```

**4. Recarregar manualmente:**
```bash
launchctl unload ~/Library/LaunchAgents/com.inlocker.backup.*.plist
launchctl load ~/Library/LaunchAgents/com.inlocker.backup.*.plist
```

---

### Restore não encontra backups

**1. Verificar se backups existem:**
```bash
ls -lh /path/to/destination/*.tar.zst
```

**2. Logs no console:**
```
[BackupList] No backups found for this configuration
```

**3. Verificar formato dos arquivos:**
- Devem terminar com `.tar.zst`
- Formato: `backup_incr_20251101_140530.tar.zst`

---

## 📝 Logs Úteis para Report de Bugs

Se encontrar um bug, inclua:

1. **Console do browser** (Cmd+Option+I):
```
Copy todos os logs com [BackupList] ou erros em vermelho
```

2. **Terminal do backend**:
```
Copy mensagens com INFO, WARN, ERROR
```

3. **Informações do sistema**:
```bash
sw_vers  # Versão do macOS
cargo --version  # Versão do Rust
node --version  # Versão do Node
```

4. **Configuração do backup**:
```json
{
  "id": "backup-123",
  "source_path": "/Users/...",
  "destination_path": "/Users/...",
  "backup_type": "incremental"
}
```

---

## 🚀 Testando um Backup Manualmente

### Passo a Passo:

1. **Abrir Developer Tools** (Cmd+Option+I)

2. **Criar backup de teste:**
   - Source: Pasta com alguns arquivos pequenos
   - Destination: Pasta vazia com permissão de escrita
   - Type: Incremental

3. **Clicar "Run Backup"**

4. **Observar:**
   - ✅ Console: `[BackupList] Starting backup...`
   - ✅ UI: Spinner azul "Backup in progress..."
   - ✅ Terminal: `INFO Running backup for: ...`
   - ✅ Terminal: `INFO Backup completed: ...`
   - ✅ UI: Caixa verde com resultado

5. **Verificar arquivo criado:**
```bash
ls -lh /path/to/destination/
# Deve mostrar: backup_incr_YYYYMMDD_HHMMSS.tar.zst
```

6. **Testar Restore:**
   - Clicar "Restore"
   - Selecionar backup da lista
   - Escolher pasta de destino
   - Confirmar

7. **Verificar restauração:**
```bash
ls -la /path/to/restore/destination/
# Deve mostrar todos os arquivos originais
```

---

## 🎯 Verificação Rápida (Checklist)

- [ ] App abre sem erros
- [ ] Console do browser não mostra erros vermelhos
- [ ] Consigo criar um backup config
- [ ] Botão "Run Backup" responde
- [ ] Vejo spinner durante backup
- [ ] Vejo resultado verde após backup
- [ ] Arquivo .tar.zst foi criado na destination
- [ ] Consigo fazer restore
- [ ] Arquivos restaurados estão corretos
- [ ] Schedule aparece em `launchctl list | grep inlocker`

---

**Se tudo isso funcionar, o InLocker está 100% operacional!** 🎉
