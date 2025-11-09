# BUG #002: Sistema de Agendamento Não Funcional

**Status:** 🔴 CRITICAL BLOCKER
**Priority:** P0 (Bloqueia produção)
**Branch:** `fix/scheduling-system-overhaul`
**Created:** 2025-11-09
**Assignee:** Claude Code

---

## Resumo do Problema

O sistema de agendamento automático de backups **não está funcionando**. Backups agendados nunca são executados automaticamente, tornando a feature principal de automação completamente quebrada.

### Sintomas

- ✅ UI permite configurar agendamento (cron expressions)
- ✅ Comando `register_schedule` executa sem erros
- ❌ Nenhum backup agendado é executado automaticamente
- ❌ Nenhum arquivo .plist criado em `~/Library/LaunchAgents/`
- ❌ Nenhum job ativo no launchd (`launchctl list | grep inlocker`)
- ❌ Nenhum log gerado em `/tmp/inlocker-*.log`

### Impacto

**BLOQUEADOR DE PRODUÇÃO**: Sem agendamento funcional, o app perde sua proposta de valor principal (backups automáticos).

---

## Diagnóstico Técnico

### Causa Raiz Identificada

1. **Arquitetura Confusa: Dois Sistemas Simultâneos**
   - `tokio-cron-scheduler` (scheduler.rs) - funciona APENAS com app aberto
   - `launchd` (launchd.rs) - deveria funcionar independentemente
   - **Resultado**: Complexidade desnecessária, nenhum funciona corretamente

2. **launchd Não Cria os Arquivos .plist**
   - Verificação do sistema: nenhum arquivo em `~/Library/LaunchAgents/com.inlocker*`
   - Possíveis causas:
     - Caminho do executável incorreto (bundle path vs binary path)
     - Falha silenciosa sem logs de erro
     - Falta de verificação pós-instalação

3. **Falta de Debugging e Validação**
   - Logs em `/tmp` são voláteis (apagados ao reiniciar)
   - Sem verificação se .plist foi criado com sucesso
   - Sem teste manual após registro (`launchctl kickstart`)
   - Feedback de erros não aparece na UI

4. **Caminho do Executável Incorreto**
   - Código atual (commands.rs:347):
     ```rust
     let app_path = std::env::current_exe()  // ❌ Aponta para bundle interno
     ```
   - Deveria ser:
     ```rust
     /Applications/InLocker.app/Contents/MacOS/inlocker  // ✅ Executável correto
     ```

---

## Solução Proposta

### Arquitetura Nova: Sistema Híbrido Robusto

```
┌─────────────────────────────────────────────┐
│  launchd (PRINCIPAL - macOS nativo)         │
│  • Backups agendados independentes          │
│  • Funciona mesmo com app fechado          │
│  • .plist em ~/Library/LaunchAgents         │
└─────────────────────────────────────────────┘
              ↓ dispara
┌─────────────────────────────────────────────┐
│  InLocker CLI Mode (--backup config_id)     │
│  • Executa backup via linha de comando      │
│  • Envia notificação macOS                  │
│  • Logs em ~/Library/Logs/InLocker/         │
└─────────────────────────────────────────────┘
```

### Mudanças Principais

1. **Simplificar para apenas launchd** (remover tokio-cron-scheduler)
2. **Corrigir caminho do executável** (bundle path vs binary path)
3. **Logs persistentes** (`~/Library/Logs/InLocker/` em vez de `/tmp`)
4. **Verificação robusta pós-instalação** (criar, carregar, verificar, testar)
5. **UI de diagnóstico** (status do agendamento, próxima execução, logs)

---

## Checklist de Implementação

### Branch e Setup
- [ ] Criar branch `fix/scheduling-system-overhaul` a partir de `main`
- [ ] Verificar que não há modificações pendentes em main

### Fase 1: Diagnóstico (30min)
- [ ] Criar comando `diagnose_schedule(config_id)` em commands.rs
- [ ] Adicionar logs detalhados em `launchd::install_launch_agent`
- [ ] Verificar se .plist está sendo criado
- [ ] Verificar se agent está sendo loaded
- [ ] Identificar exatamente onde está falhando

### Fase 2: Fix launchd (2-3h)

#### 2.1 Corrigir Caminho do Executável
- [ ] Modificar `commands.rs:register_schedule`
- [ ] Detectar se está em dev mode ou production bundle
- [ ] Dev mode: usar `std::env::current_exe()`
- [ ] Production: usar `/Applications/InLocker.app/Contents/MacOS/inlocker`
- [ ] Adicionar log do caminho usado

#### 2.2 Logs Persistentes
- [ ] Modificar `launchd.rs:generate_plist_content`
- [ ] Mudar StandardOutPath de `/tmp` para `~/Library/Logs/InLocker/`
- [ ] Criar diretório de logs se não existir
- [ ] Formato: `scheduled-{config_id}-YYYY-MM-DD.log`

#### 2.3 Verificação Robusta Pós-Instalação
- [ ] Modificar `launchd::install_launch_agent`
- [ ] Adicionar: verificar se .plist foi criado
- [ ] Adicionar: verificar se agent aparece em `launchctl list`
- [ ] Adicionar: teste manual com `launchctl kickstart`
- [ ] Retornar erro detalhado se qualquer passo falhar

#### 2.4 Comando de Diagnóstico
- [ ] Criar struct `ScheduleDiagnostics` em types.rs
- [ ] Implementar `diagnose_schedule` command
- [ ] Verificar: .plist existe?
- [ ] Verificar: Agent está loaded?
- [ ] Verificar: Próxima execução agendada?
- [ ] Verificar: Logs existem e são acessíveis?
- [ ] Verificar: Permissões do executável

### Fase 3: Remover tokio-cron-scheduler (1h)
- [ ] Remover `tokio-cron-scheduler` de Cargo.toml
- [ ] Remover ou simplificar scheduler.rs
- [ ] Atualizar `commands.rs:register_schedule` (remover chamada ao in-app scheduler)
- [ ] Atualizar `lib.rs` (remover inicialização do SchedulerState ou simplificar)
- [ ] Atualizar tech-stack.md
- [ ] Executar `cargo check` e `cargo clippy`

### Fase 4: UI de Diagnóstico (1h)

#### 4.1 Backend Commands
- [ ] Adicionar `get_next_scheduled_execution(config_id)` command
- [ ] Adicionar `test_schedule_now(config_id)` command (launchctl kickstart)
- [ ] Adicionar `open_schedule_logs(config_id)` command (abre Finder)

#### 4.2 Frontend UI
- [ ] Adicionar botão "Test Schedule Now" no BackupList
- [ ] Mostrar próxima execução agendada
- [ ] Adicionar link "View Logs" que abre diretório de logs
- [ ] Mostrar status: "Scheduled ✓" ou "Schedule Error ⚠️"
- [ ] Adicionar toast de feedback ao testar agendamento

### Fase 5: Testes e Validação (1-2h)

#### 5.1 Testes Manuais
- [ ] Dev mode: Configurar agendamento para daqui a 2 minutos
- [ ] Verificar que .plist foi criado
- [ ] Verificar que agent aparece em `launchctl list`
- [ ] Aguardar execução agendada
- [ ] Verificar que backup foi executado
- [ ] Verificar logs em `~/Library/Logs/InLocker/`
- [ ] Verificar notificação foi enviada

#### 5.2 Testes Production Build
- [ ] Build production: `pnpm tauri build`
- [ ] Instalar .dmg gerado
- [ ] Configurar agendamento
- [ ] Testar execução agendada
- [ ] Verificar caminho do executável está correto

#### 5.3 Testes Edge Cases
- [ ] Testar agendamento com app fechado
- [ ] Testar múltiplos agendamentos simultâneos
- [ ] Testar remoção de agendamento
- [ ] Testar edição de agendamento existente
- [ ] Testar sistema após reboot do macOS

### Fase 6: Documentação e Limpeza (30min)
- [ ] Atualizar roadmap.md (marcar Fase 3 como completa)
- [ ] Atualizar CLAUDE.md com nova arquitetura
- [ ] Adicionar comentários no código sobre launchd
- [ ] Atualizar tech-stack.md
- [ ] Criar commit descritivo
- [ ] Abrir PR para main

---

## Arquivos a Modificar

### Backend (Rust)
- `src-tauri/src/launchd.rs` - Fix caminho executável, logs persistentes, verificação
- `src-tauri/src/commands.rs` - Adicionar diagnose_schedule, melhorar register_schedule
- `src-tauri/src/types.rs` - Adicionar ScheduleDiagnostics struct
- `src-tauri/src/lib.rs` - Simplificar ou remover SchedulerState
- `src-tauri/src/scheduler.rs` - Remover ou simplificar drasticamente
- `src-tauri/Cargo.toml` - Remover tokio-cron-scheduler

### Frontend (React/TypeScript)
- `src/ui/components/BackupList.tsx` - Adicionar UI de diagnóstico
- `src/ui/store/useBackupStore.ts` - Adicionar estados de diagnóstico

### Documentação
- `docs/01-planning/03-tech-stack.md` - Remover tokio-cron-scheduler
- `docs/02-development/01-roadmap.md` - Atualizar status Fase 3
- `CLAUDE.md` - Atualizar arquitetura de agendamento

---

## Critérios de Sucesso

- [ ] .plist criado com sucesso em `~/Library/LaunchAgents/`
- [ ] Agent aparece em `launchctl list`
- [ ] Backup agendado executa automaticamente no horário correto
- [ ] Backup executa mesmo com app fechado
- [ ] Logs aparecem em `~/Library/Logs/InLocker/`
- [ ] Notificação enviada ao completar backup agendado
- [ ] UI mostra status correto do agendamento
- [ ] Comando "Test Schedule Now" funciona
- [ ] Sistema sobrevive a reboot do macOS
- [ ] 0 dependências desnecessárias (tokio-cron-scheduler removido)

---

## Referências

### Research realizada
- macOS launchd best practices 2025
- Tauri app scheduling integration
- launchd debugging techniques

### Documentação oficial
- [Apple: Scheduling Timed Jobs](https://developer.apple.com/library/archive/documentation/MacOSX/Conceptual/BPSystemStartup/Chapters/ScheduledJobs.html)
- [launchd.plist man page](https://www.manpagez.com/man/5/launchd.plist/)
- [Tauri: macOS Application Bundle](https://v2.tauri.app/distribute/macos-application-bundle/)

### Issues relacionados
- BUG #001: Restore button not responding (resolvido)
- Roadmap Phase 3: Automation and security (bloqueado por este bug)

---

## Notas de Implementação

### Caminho do Executável (Production vs Dev)

**Dev mode:**
```rust
/Users/blc/Dev/Apps/InLocker/src-tauri/target/debug/inlocker
```

**Production bundle:**
```rust
/Applications/InLocker.app/Contents/MacOS/inlocker
```

**Detecção:**
```rust
fn get_executable_path() -> Result<String, String> {
    let current = std::env::current_exe()
        .map_err(|e| format!("Failed to get current exe: {}", e))?;

    // Check if running from bundle
    if let Some(path_str) = current.to_str() {
        if path_str.contains(".app/Contents/MacOS") {
            // Production: use bundle path
            return Ok("/Applications/InLocker.app/Contents/MacOS/inlocker".to_string());
        }
    }

    // Dev mode: use current executable
    Ok(current.to_string_lossy().to_string())
}
```

### Formato do .plist

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>com.inlocker.backup.{config_id}</string>

  <key>ProgramArguments</key>
  <array>
    <string>/Applications/InLocker.app/Contents/MacOS/inlocker</string>
    <string>--backup</string>
    <string>{config_id}</string>
  </array>

  <key>StartCalendarInterval</key>
  <dict>
    <key>Hour</key>
    <integer>14</integer>
    <key>Minute</key>
    <integer>30</integer>
  </dict>

  <key>RunAtLoad</key>
  <false/>

  <key>StandardOutPath</key>
  <string>{home}/Library/Logs/InLocker/scheduled-{config_id}.log</string>

  <key>StandardErrorPath</key>
  <string>{home}/Library/Logs/InLocker/scheduled-{config_id}.err</string>
</dict>
</plist>
```

### Comandos de Verificação

```bash
# Listar agents do InLocker
ls -la ~/Library/LaunchAgents/com.inlocker*

# Verificar se agent está loaded
launchctl list | grep inlocker

# Ver detalhes do agent
launchctl print gui/$(id -u)/com.inlocker.backup.{config_id}

# Teste manual
launchctl kickstart -k gui/$(id -u)/com.inlocker.backup.{config_id}

# Ver logs
tail -f ~/Library/Logs/InLocker/scheduled-*.log
```

---

## Timeline Estimado

| Fase | Duração | Descrição |
|------|---------|-----------|
| Setup + Branch | 5min | Criar branch e preparar ambiente |
| Fase 1: Diagnóstico | 30min | Identificar falha exata |
| Fase 2: Fix launchd | 2-3h | Implementar correções principais |
| Fase 3: Remove scheduler | 1h | Simplificar arquitetura |
| Fase 4: UI diagnóstico | 1h | Feedback visual |
| Fase 5: Testes | 1-2h | Validação completa |
| Fase 6: Docs | 30min | Documentação e limpeza |
| **TOTAL** | **6-8h** | Implementação completa |

---

## Riscos e Mitigações

| Risco | Probabilidade | Impacto | Mitigação |
|-------|--------------|---------|-----------|
| Caminho executável ainda incorreto | Médio | Alto | Adicionar logs detalhados, testar em prod |
| Permissões do launchd | Baixo | Alto | Verificar com launchctl print |
| Regressão em funcionalidades | Baixo | Médio | Testes manuais extensivos |
| Build production quebrado | Baixo | Alto | Testar build antes de merge |

---

**Última atualização:** 2025-11-09
**Autor:** Claude Code (solicitado por usuário)
