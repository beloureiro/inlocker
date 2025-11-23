# BUG #002: Sistema de Agendamento Não Funcional

**Status:** 🔴 NÃO RESOLVIDO - TELA BRANCA
**Priority:** P0 (Bloqueia produção)
**Branch:** `fix/scheduling-system-overhaul`
**Created:** 2025-11-09
**Last Updated:** 2025-11-23 (Tela branca sem UI aparece)
**Progress:** [ ] Tela branca aparece, UI não carrega

---

## 📊 Progresso Atual

### ✅ Completo (Fases 1-4.5 - 90%)
- Diagnóstico implementado
- Logs persistentes funcionando
- Caminho do executável corrigido
- Verificação robusta (9 passos)
- Testes automatizados passando (2/2)
- **tokio-cron-scheduler removido** ✅
- **Arquitetura simplificada (apenas launchd)** ✅
- **UI de diagnóstico implementada** ✅
  - Botão "Test Now" para executar backup agendado manualmente
  - Botão "Logs" para abrir diretório de logs no Finder
- **UI de agendamento simplificada** ✅
  - Removido campo de cron expression customizado
  - Adicionados seletores simples: Hour, Minute, Day of Week, Day of Month
  - Resumo em linguagem natural (ex: "Runs daily at 14:00")
  - Cron expression gerado internamente (invisível ao usuário)
- **CLI Mode PARCIALMENTE implementado** ⚠️
  - Parse de argumentos `--backup <config_id>` (lib.rs:28-36) ✅
  - ❌ **BUG:** Execução ainda abre janela principal (lib.rs:92-106)
  - Função completa `run_scheduled_backup` (lib.rs:122-234) ✅
  - Notificações macOS ao completar ✅
  - Exit codes corretos (0=sucesso, 1=erro) ✅
  - **PROBLEMA:** Backup executa DEPOIS do `tauri::Builder`, então GUI sempre inicializa
- Compilação limpa com 0 erros ✅

### 🆕 Implementações 2025-11-14

**Correção macOS 26 Tahoe (comandos deprecated):**
- [x] Migrado `launchctl load/unload` para `bootstrap/bootout` em `launchd.rs:391-405`
- [x] Migrado `unload` para `bootout` em `launchd.rs:490-502`
- [x] Atualizado `install_launch_agent()` para usar comandos modernos
- [x] Atualizado `uninstall_launch_agent()` para usar comandos modernos
- [x] Teste manual confirmou: backup dispara automaticamente no horário agendado

**UI de Progresso para Backups Agendados:**
- [x] Criado componente `ScheduledBackupProgress.tsx` com design customizado
- [x] Barra de progresso animada (0-100%)
- [x] Mensagens de status em português (inicializando, escaneando, comprimindo, finalizando)
- [x] Contador de arquivos processados
- [x] Ícone animado de loading

**Detecção de Modo CLI - Tentativa 1 (comando customizado - FALHOU):**
- [x] Criado comando Tauri `is_scheduled_mode()` em `commands.rs:520-525` - não funcionou
- [x] Registrado comando em `lib.rs:86` - não funcionou
- [x] Frontend detecta modo CLI via comando Tauri - não funcionou, tela branca

**Eventos de Progresso Backend → Frontend:**
- [x] Adicionado import `use tauri::{Emitter, Manager}` em `lib.rs:11`
- [x] Evento "initializing" com 0% em `lib.rs:155-159`
- [x] Evento "scanning" com 10% em `lib.rs:169-173`
- [x] Evento "compressing" com 30% em `lib.rs:198-202`
- [x] Evento "finalizing" com 90% em `lib.rs:219-223`
- [x] Evento "completed" com 100% em `lib.rs:243-249`
- [x] Frontend escuta evento `backup-progress` em `ScheduledBackupProgress.tsx:24-28`

**Compilação:**
- [x] `cargo check` passa com 0 erros (4 warnings de código não usado - aceitável)

**Detecção de Modo CLI - Tentativa 2 (plugin oficial - FALHOU):**
- [x] Instalado `@tauri-apps/plugin-cli` (pnpm) e `tauri-plugin-cli` (cargo)
- [x] Plugin registrado em `lib.rs:66` com `.plugin(tauri_plugin_cli::init())`
- [x] Configurado argumento `--backup` em `tauri.conf.json:12-23`
- [x] Adicionada permissão `cli:default` em `capabilities/default.json:8`
- [x] Frontend atualizado para usar `getMatches()` oficial em `App.tsx:16-32`
- [x] Estado inicial `null` + loading azul (`App.tsx:73-82`) - ainda tela branca

**Tentativa 3 (visible:false + show programático - AGUARDANDO TESTE):**
- [x] Configurado `"visible": false` na janela principal (`tauri.conf.json:36`)
- [x] Backend mostra janela quando pronto (`lib.rs:98-100` CLI mode e `lib.rs:118-120` normal)
- [x] Compilação: 0 erros
- [ ] Teste usuário: verificar se eliminou tela branca

### ❌ Testes Falharam
- [ ] Teste: backup dispara automaticamente no horário configurado
- [ ] Teste: janela mostra UI customizada (não tela branca) - **FALHOU: TELA BRANCA**
- [ ] Teste: progresso atualiza em tempo real
- [ ] Teste: notificação macOS ao completar
- [ ] Teste: janela fecha automaticamente após conclusão
- [ ] Build e teste de produção (.dmg)
- [ ] Atualização do roadmap após confirmação

---

## Resumo do Problema

O sistema de agendamento **dispara backups corretamente**, mas **abre segunda janela do app** ao executar backup agendado, criando uma UX ruim e confusão para o usuário.

### Sintomas Atuais (2025-11-21)

- ✅ UI permite configurar agendamento (interface simplificada com seletores de horário)
- ✅ Comando `register_schedule` executa sem erros
- ✅ Arquivo .plist criado corretamente em `~/Library/LaunchAgents/`
- ✅ Job ativo no launchd (`launchctl list | grep inlocker`)
- ✅ Backup agendado DISPARA automaticamente no horário correto
- ✅ Logs gerados em `~/Library/Logs/InLocker/`
- ❌ **BUG ATIVO:** Segunda janela do app abre quando backup agendado executa
- ❌ **BUG ATIVO:** Se app já está aberto, abre instância duplicada (confunde usuário)
- ❌ Janela de backup agendado deveria ser SEPARADA da janela principal do app

### Arquitetura Esperada: DUAS JANELAS DIFERENTES

**JANELA 1: Principal do App (uso diário)**
- Configuração de backups
- Agendamento de schedules
- Lista de backups salvos
- Botão "Run Backup" manual
- Esta janela NÃO deve ser duplicada

**JANELA 2: Progresso de Backup Agendado (launchd dispara)**
- Aparece APENAS quando launchd executa backup agendado
- Mostra progresso em tempo real
- Fecha automaticamente ao completar
- Independente da janela principal
- Deve funcionar mesmo se janela principal estiver fechada

### Impacto

**BLOQUEADOR DE PRODUÇÃO**: UX ruim, usuário vê duplicação de janelas e fica confuso sobre o que está acontecendo.

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

4. **Caminho do Executável Incorreto** ✅ RESOLVIDO
   - Código atual (commands.rs:347):
     ```rust
     let app_path = std::env::current_exe()  // ❌ Aponta para bundle interno
     ```
   - Deveria ser:
     ```rust
     /Applications/InLocker.app/Contents/MacOS/inlocker  // ✅ Executável correto
     ```

5. **launchd Não Recarrega Após Edição de Schedule** 🔴 CONFIRMADO (2025-11-09)
   - **Problema**: Quando usuário EDITA um schedule existente, o código atualiza o arquivo `.plist` mas o `launchd` continua usando a configuração antiga em memória
   - **Evidência**:
     ```bash
     # Arquivo .plist no disco
     Hour: 17, Minute: 9

     # launchd em memória (usando configuração antiga!)
     Hour: 16, Minute: 13
     ```
   - **Teste realizado**:
     ```bash
     # ANTES: launchd mostrava 16:13 (configuração antiga)
     launchctl print gui/$(id -u)/com.inlocker.backup.xxx

     # Após unload + load manual
     launchctl unload ~/Library/LaunchAgents/com.inlocker.backup.xxx.plist
     launchctl load ~/Library/LaunchAgents/com.inlocker.backup.xxx.plist

     # DEPOIS: launchd mostrava 17:09 (configuração atualizada!) ✅
     ```
   - **Causa**: Função `install_launch_agent()` em `launchd.rs` NÃO faz `unload` antes de `load` quando atualiza schedule existente
   - **Impacto**: Usuário edita horário (ex: para daqui a 5 minutos) mas o backup NÃO executa porque launchd ainda usa horário antigo
   - **Solução**: Modificar `install_launch_agent()` para sempre fazer `unload` + `load` (ou usar `bootout` + `bootstrap` no macOS moderno)

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

**STATUS ATUAL**: Fases 1-4.5 completas ✅ | Fases 5-6 pendentes ⏸️

### Branch e Setup
- [x] Criar branch `fix/scheduling-system-overhaul` a partir de `main`
- [x] Verificar que não há modificações pendentes em main

### Fase 1: Diagnóstico (30min) ✅ COMPLETA
- [x] Criar comando `diagnose_schedule(config_id)` em commands.rs
- [x] Adicionar logs detalhados em `launchd::install_launch_agent`
- [x] Verificar se .plist está sendo criado
- [x] Verificar se agent está sendo loaded
- [x] Identificar exatamente onde está falhando
- [x] Criar testes automatizados de integração (EXTRA)

### Fase 2: Fix launchd (2-3h) ✅ COMPLETA

#### 2.1 Corrigir Caminho do Executável ✅
- [x] Modificar `commands.rs:register_schedule`
- [x] Detectar se está em dev mode ou production bundle
- [x] Dev mode: usar `std::env::current_exe()`
- [x] Production: usar `/Applications/InLocker.app/Contents/MacOS/inlocker`
- [x] Adicionar log do caminho usado
- [x] Criar função `get_executable_path()` em launchd.rs (EXTRA)

#### 2.2 Logs Persistentes ✅
- [x] Modificar `launchd.rs:generate_plist_content`
- [x] Mudar StandardOutPath de `/tmp` para `~/Library/Logs/InLocker/`
- [x] Criar diretório de logs se não existir
- [x] Formato: `scheduled-{config_id}.log`
  - **NOTA**: Sem timestamp no nome (mais simples, sobrescreve)
- [x] Adicionar funções `get_log_path()` e `get_error_log_path()` (EXTRA)

#### 2.3 Verificação Robusta Pós-Instalação ✅
- [x] Modificar `launchd::install_launch_agent`
- [x] Adicionar: verificar se .plist foi criado
- [x] Adicionar: verificar se agent aparece em `launchctl list`
- [x] Adicionar: teste manual com `launchctl kickstart`
- [x] Retornar erro detalhado se qualquer passo falhar
- [x] Implementar verificação em 9 passos com logs detalhados (EXTRA)

#### 2.4 Comando de Diagnóstico ✅
- [x] Criar struct `ScheduleDiagnostics` em types.rs
- [x] Implementar `diagnose_schedule` command
- [x] Verificar: .plist existe?
- [x] Verificar: Agent está loaded?
- [x] Verificar: Próxima execução agendada?
- [x] Verificar: Logs existem e são acessíveis?
- [x] Verificar: Permissões do executável
- [x] Adicionar função `is_agent_loaded()` (EXTRA)
- [x] Adicionar função `get_user_uid()` (EXTRA)
- [x] Registrar comando em lib.rs (EXTRA)

### ✅ Testes Automatizados (EXTRA - Implementado)
- [x] Criar `tests/scheduling_system_tests.rs`
- [x] Teste: `test_scheduling_system_complete_workflow`
  - Testa criação de .plist, load no launchctl, kickstart
- [x] Teste: `test_launchd_helper_functions`
  - Testa funções auxiliares (path, HOME, UID, launchctl)
- [x] **Resultado**: 2 testes passando, 0 falhando
- [x] Confirmar que infraestrutura funciona

### Fase 3: Remover tokio-cron-scheduler (1h) ✅ COMPLETA
- [x] Remover `tokio-cron-scheduler` de Cargo.toml
- [x] Simplificar scheduler.rs (mantido como placeholder)
- [x] Atualizar `commands.rs:register_schedule` (removida chamada ao in-app scheduler)
- [x] Atualizar `commands.rs:unregister_schedule`
- [x] Atualizar `commands.rs:check_schedule_status` (usa launchd agora)
- [x] Manter SchedulerState em lib.rs (compatibilidade)
- [ ] Atualizar tech-stack.md ⏸️ (Fase 6)
- [x] Executar `cargo check` (0 erros, 3 warnings aceitáveis)
- [x] Executar testes (2/2 passando)

### Fase 4: UI de Diagnóstico (1h) ✅ COMPLETA

#### 4.1 Backend Commands ✅
- [x] Adicionar `test_schedule_now(config_id)` command (launchctl kickstart)
- [x] Adicionar `open_schedule_logs(config_id)` command (abre Finder)
- [x] Registrar comandos em lib.rs

#### 4.2 Frontend UI ✅
- [x] Adicionar botão "Test Now" no BackupList
  - Apenas visível quando schedule está ativo
  - Executa kickstart manual do launchd
  - Mostra alert com resultado
- [x] Adicionar botão "Logs" que abre diretório de logs no Finder
- [x] Badge visual de schedule já existe (ícone de relógio)
- [ ] Mostrar próxima execução agendada ⏸️ (future enhancement)
- [ ] Toast notifications ⏸️ (usando alerts por enquanto)

#### 4.3 UI Simplificada (Remover Cron Exposure) ✅ COMPLETA
- [x] Remover campo "Custom Schedule" do dropdown
- [x] Remover input de cron expression com documentação
- [x] Adicionar seletores simples de Time (Hour 0-23, Minute 0-59)
- [x] Adicionar seletor Day of Week para preset "Weekly"
- [x] Adicionar seletor Day of Month para preset "Monthly"
- [x] Adicionar resumo visual em linguagem natural
  - "Runs every hour"
  - "Runs daily at 14:00"
  - "Runs every Monday at 14:00"
  - "Runs on day 1 of each month at 14:00"
- [x] Gerar cron expression internamente (não expor ao usuário)
- [x] Atualizar BackupList.tsx para mostrar presets em vez de cron
- [x] Remover função `formatCronExpression()` obsoleta

### Fase 4.5: CLI Mode Implementation (1-2h) ❌ INCOMPLETA - BUG ATIVO
- [x] Implementar parse de argumentos CLI em `src-tauri/src/main.rs` ou `lib.rs`
- [x] Detectar flag `--backup <config_id>` nos argumentos do processo
- [ ] **BUG ATIVO:** Executar backup sem abrir janela da UI (modo headless) - AINDA ABRE JANELA PRINCIPAL
- [x] Carregar configuração do backup pelo config_id
- [x] Executar lógica de backup (comprimir, encriptar, salvar)
- [x] Enviar notificação macOS ao completar
- [x] Escrever output para stdout/stderr (capturado pelo launchd)
- [x] Sair do processo após completar (exit code 0 = sucesso, 1 = erro)
- [ ] Testar manualmente: `/path/to/inlocker --backup test-id` ⏸️ (Fase 5)

**PROBLEMA ATUAL:**
- Código executa backup DEPOIS do `tauri::Builder` (lib.rs:92-115)
- `tauri::Builder` sempre inicializa GUI completa (webview, plugins, janela principal)
- `window.show()` é chamado explicitamente (lib.rs:99)
- Resultado: Segunda instância do app abre quando launchd dispara backup agendado

**SOLUÇÃO NECESSÁRIA:**
- [ ] Executar backup ANTES do `tauri::Builder` (true headless)
- [ ] OU criar janela SEPARADA para progresso de backup agendado (não usar janela principal)
- [ ] Implementar `tauri-plugin-single-instance` para prevenir múltiplas instâncias da janela principal

---

### Fase 4.6: Correção - Janelas Separadas (1-2h) ✅ IMPLEMENTADO - NÃO TESTADO

**O QUE JÁ EXISTE:**
- ✅ Componente `ScheduledBackupProgress.tsx` criado
- ✅ `App.tsx` detecta modo CLI e renderiza componente correto
- ✅ `lib.rs` detecta `--backup` args
- ✅ `tauri-plugin-cli` instalado e configurado

**O QUE FALTA (CORREÇÃO DO BUG):**

#### 4.6.1 Configurar Segunda Janela (30min) ✅ CONCLUÍDO
- [x] Editar `src-tauri/tauri.conf.json`
- [x] Adicionar segunda janela com label "scheduled-progress":
  ```json
  "windows": [
    {
      "label": "main",
      "title": "InLocker",
      "width": 1400,
      "height": 900,
      "visible": false
    },
    {
      "label": "scheduled-progress",
      "title": "Backup Agendado",
      "width": 600,
      "height": 400,
      "center": true,
      "resizable": false,
      "visible": false
    }
  ]
  ```

#### 4.6.2 Adicionar Single Instance Plugin (15min) ✅ CONCLUÍDO
- [x] Adicionar ao `Cargo.toml`: `tauri-plugin-single-instance = "2.0.0"`
- [x] Executar: `cd src-tauri && cargo update` (instalado v2.3.6)
- [x] Registrar plugin PRIMEIRO em `lib.rs` (antes de outros plugins)
- [x] Callback deve focar janela "main" se já existir

#### 4.6.3 Modificar lib.rs - Abrir Janela Correta (30min) ✅ CONCLUÍDO
- [x] Modificar `lib.rs::setup` (linhas 92-127)
- [x] CLI mode deve abrir janela "scheduled-progress" (NÃO "main")
- [x] Normal mode deve abrir janela "main" (NÃO "scheduled-progress")
- [x] Adicionar plugin single-instance como PRIMEIRO plugin
- [x] Callback para focar janela main se já existir

#### 4.6.4 Atualizar App.tsx - Detectar Janela Correta (15min) ✅ CONCLUÍDO
- [x] Verificar se `App.tsx` precisa mudanças → **NÃO precisa!**
- [x] Componente `ScheduledBackupProgress` já renderiza corretamente
- [x] App.tsx funciona para ambas as janelas (detecta modo CLI automaticamente)

#### 4.6.5 Testar Correção (30min)
- [ ] Teste 1: Abrir app normal → deve abrir janela "main"
- [ ] Teste 2: Executar `--backup` com app fechado → deve abrir janela "scheduled-progress" APENAS
- [ ] Teste 3: App "main" aberto + `--backup` dispara → "scheduled-progress" abre, "main" continua
- [ ] Teste 4: Tentar abrir app duas vezes → single instance previne duplicação de "main"
- [ ] Teste 5: launchd dispara backup → janela "scheduled-progress" aparece, fecha ao terminar
- [ ] **BUG ATIVO**: Tela branca aparece sem UI

#### 4.6.6 Tentativa: on_page_load - FALHOU
- [ ] Tentado: .on_page_load() no Builder com PageLoadEvent::Finished
- [ ] Resultado: janelas não aparecem na tela (mesmo log dizendo sucesso)
- [ ] Problema: on_page_load detecta carregamento do HTML mas React ainda não renderizou
- [ ] Revertido com git restore

#### 4.6.7 Solução correta: evento "window-ready" do frontend - IMPLEMENTADO
- [x] Frontend emite evento quando React termina render (App.tsx useEffect)
- [x] Backend escuta evento e aí chama show() (lib.rs listeners)
- [x] Explica porque Test Now funciona (React já renderizou)
- [x] import Listener trait no lib.rs
- [ ] TESTAR: pnpm tauri dev e launchd trigger

---

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
- `src/ui/components/BackupList.tsx` - Adicionar UI de diagnóstico ✅ | Simplificar exibição de schedule ✅
- `src/ui/components/BackupConfigModal.tsx` - Simplificar UI de agendamento (remover cron exposure) ✅
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

| Fase | Duração | Status | Descrição |
|------|---------|--------|-----------|
| Setup + Branch | 5min | ✅ | Criar branch e preparar ambiente |
| Fase 1: Diagnóstico | 30min | ✅ | Identificar falha exata |
| Fase 2: Fix launchd | 2-3h | ✅ | Implementar correções principais |
| Fase 3: Remove scheduler | 1h | ✅ | Simplificar arquitetura |
| Fase 4: UI diagnóstico | 1h | ✅ | Feedback visual |
| Fase 4.5: CLI Mode | 1-2h | ✅ | Parse args, exec headless, notificações |
| Fase 5: Testes | 1-2h | ⏸️ | Validação completa |
| Fase 6: Docs | 30min | ⏸️ | Documentação e limpeza |
| **TOTAL** | **6-8h** | **90%** | ~1.5-2h restantes |

---

## Riscos e Mitigações

| Risco | Probabilidade | Impacto | Mitigação |
|-------|--------------|---------|-----------|
| Caminho executável ainda incorreto | Médio | Alto | Adicionar logs detalhados, testar em prod |
| Permissões do launchd | Baixo | Alto | Verificar com launchctl print |
| Regressão em funcionalidades | Baixo | Médio | Testes manuais extensivos |
| Build production quebrado | Baixo | Alto | Testar build antes de merge |

---

## 🎯 Próximos Passos Obrigatórios

### OPÇÃO A: Continuar Implementação (Recomendado)

**Fase 5: Testes Manuais (1-2h)**
1. [ ] Executar `pnpm tauri dev`
2. [ ] Configurar agendamento de teste
3. [ ] Aguardar execução agendada ou usar `test_schedule_now`
4. [ ] Verificar logs em `~/Library/Logs/InLocker/`
5. [ ] Verificar notificações macOS
6. [ ] Testar CLI mode manualmente: `/path/to/inlocker --backup test-id`

**Fase 6: Build e Documentação (30min)**
1. [ ] Build production: `pnpm tauri build`
2. [ ] Testar .dmg instalado em `/Applications`
3. [ ] Verificar caminho do executável está correto
4. [ ] Atualizar documentação (roadmap, tech-stack)
5. [ ] Commit e PR

**Tempo Total Restante:** ~1.5-2.5 horas

---

### OPÇÃO B: Testar Estado Atual

**Teste Manual Rápido (10min)**

```bash
# 1. Executar app em dev mode
rm -rf dist node_modules/.vite && pnpm tauri dev

# 2. No DevTools console:
await window.__TAURI__.invoke('diagnose_schedule', { configId: 'seu-config-id' })

# 3. Verificar resultado do diagnóstico
```

**Verificar manualmente:**
- [ ] .plist foi criado em `~/Library/LaunchAgents/`
- [ ] Agent aparece em `launchctl list | grep inlocker`
- [ ] Logs em `~/Library/Logs/InLocker/`

---

### OPÇÃO C: Comitar Progresso Parcial

**Commit Fase 1-4.5 (90% completo)**
- ✅ Infraestrutura backend completa
- ✅ Testes automatizados passando (2/2)
- ✅ UI de diagnóstico e agendamento simplificada
- ✅ CLI Mode implementado
- ⏸️ Validação manual e documentação pendentes

**Branch:** `fix/scheduling-system-overhaul`
**Merge:** Aguardar conclusão de Fase 5-6 (testes finais)

---

**Última atualização:** 2025-11-23
**Autor:** Claude Code
**Revisão:** [ ] Tela branca - solução: on_page_load
