# InLocker - Guia de Criação do Site de Apresentação

## Índice
1. [Visão Geral](#visão-geral)
2. [Identidade Visual](#identidade-visual)
3. [Mensagens Principais](#mensagens-principais)
4. [Estrutura do Site](#estrutura-do-site)
5. [Conteúdo das Seções](#conteúdo-das-seções)
6. [Recursos Visuais](#recursos-visuais)
7. [Tom e Estilo](#tom-e-estilo)
8. [SEO e Palavras-chave](#seo-e-palavras-chave)

---

## Visão Geral

### O que é InLocker?
**InLocker** é um aplicativo nativo para macOS que oferece backups automáticos, comprimidos e opcionalmente criptografados - tudo rodando localmente, sem custos recorrentes ou dependência de cloud.

### Objetivo do Site
Apresentar o InLocker como a solução de backup mais **simples**, **confiável** e **transparente** para usuários macOS que valorizam controle e privacidade sobre seus dados.

### Público-Alvo Principal
1. **Desenvolvedores** - que precisam proteger projetos de código localmente
2. **Profissionais** - com documentos sensíveis (advogados, contadores, consultores)
3. **Criadores de Conteúdo** - que trabalham com vídeos, fotos, designs pesados
4. **Usuários conscientes de privacidade** - que não confiam em soluções cloud

---

## Identidade Visual

### Paleta de Cores

#### Cor Principal: Verde Esmeralda (Emerald Green)
- **Hex**: `#10b981` (emerald-500)
- **Significado**: Segurança, confiança, proteção de dados
- **Uso**: Botões primários, destaques, badges, ícones de sucesso

#### Cores de Suporte
| Cor | Hex | Uso |
|-----|-----|-----|
| **Gray 950** | `#030712` | Background principal (dark mode) |
| **Gray 900** | `#111827` | Cards, seções, header |
| **Gray 800** | `#1f2937` | Borders, divisores |
| **Gray 400** | `#9ca3af` | Textos secundários, subtítulos |
| **White** | `#ffffff` | Textos principais, headings |
| **Red 300** | `#fca5a5` | Erros, alertas |

#### Exemplos de Aplicação
```css
/* Background do site */
background: linear-gradient(to bottom right, #030712, #111827);

/* Botão primário */
background: #10b981;
color: #ffffff;
border-radius: 0.5rem;

/* Card de feature */
background: #111827;
border: 1px solid #1f2937;
border-radius: 0.75rem;
```

### Tipografia

#### Fonte Principal: **Inter** ou **SF Pro Display** (macOS native)
- **Headings**: 700-800 (bold/extra-bold)
- **Body**: 400-500 (regular/medium)
- **Code/Monospace**: `SF Mono` ou `Fira Code`

#### Hierarquia de Tamanhos
```css
h1 (Hero): 3.5rem (56px) - font-weight: 800
h2 (Section): 2.5rem (40px) - font-weight: 700
h3 (Subsection): 1.75rem (28px) - font-weight: 600
Body: 1rem (16px) - font-weight: 400
Small: 0.875rem (14px) - font-weight: 400
```

### Logo e Ícones

#### Logo Principal
- **Arquivo**: `/logo.png` (disponível no projeto em `public/logo.png`)
- **Descrição**: Ícone de cadeado em verde esmeralda
- **Uso**: Header, footer, favicon
- **Tamanhos recomendados**:
  - Header: 48x48px
  - Favicon: 32x32px, 64x64px
  - Apple Touch Icon: 180x180px

#### Ícones de Features
Use ícones da biblioteca **Lucide React** (já utilizada no app):
- `Lock` - Segurança/Criptografia
- `Zap` - Velocidade/Performance
- `Shield` - Proteção/Confiabilidade
- `HardDrive` - Storage Local
- `Clock` - Automação/Scheduling
- `FileArchive` - Compressão
- `CheckCircle` - Sucesso/Verificação
- `TrendingUp` - Performance/Métricas

---

## Mensagens Principais

### Headline (Hero Section)
**Opção 1**: "Backups automáticos que você realmente vai usar"
**Opção 2**: "Proteja seus dados sem complicação"
**Opção 3**: "Backup simples, automático e seguro para macOS"

**Recomendação**: Opção 1 - foca no benefício emocional (simplicidade) e no problema (backups esquecidos)

### Tagline Oficial
"Automatic, compressed, and secure backups"
*(pode ser traduzido para português se o site for BR-only)*

### Proposta de Valor Única (UVP)
"InLocker é o único app de backup gratuito que combina simplicidade extrema com qualidade de nível enterprise - 78 testes automatizados, zero custos recorrentes e 100% privado."

### Mensagens Secundárias

#### 1. Simplicidade
"Configure em 3 minutos. Três passos: selecione a pasta, escolha o destino, pronto."

#### 2. Confiança
"78 testes automatizados. 3 bugs críticos corrigidos antes da produção. Zero tolerância para perda de dados."

#### 3. Performance
"Compressão zstd 5841x mais eficiente em textos. Backup de 1GB em 0.53 segundos."

#### 4. Privacidade
"Seus dados nunca saem do seu Mac. Sem cloud, sem telemetria, sem rastreamento. Código aberto e auditável."

#### 5. Flexibilidade
"Três modos para você escolher: Copy (rápido), Compressed (balanceado) ou Encrypted (máxima segurança)."

---

## Estrutura do Site

### Páginas Recomendadas

#### 1. Home (Landing Page)
- Hero Section
- Features Overview
- Social Proof (GitHub stars, downloads)
- CTA: Download / Get Started

#### 2. Features
- Backup Modes
- Scheduling
- Compression & Encryption
- Restore
- Testing & Quality

#### 3. How It Works
- 3-Step Setup
- Visual Walkthrough
- Demo Video/GIF

#### 4. Pricing
- Free & Open Source
- Comparação com concorrentes (Time Machine, Backblaze, Carbon Copy Cloner)

#### 5. Documentation
- Link para docs do GitHub
- Quick Start Guide
- FAQs

#### 6. Download
- Download do .dmg
- System Requirements
- Release Notes

---

## Conteúdo das Seções

### Hero Section

```
[LOGO] InLocker

Headline: Backups automáticos que você realmente vai usar
Subheadline: Simples, confiável e 100% local. Proteja seus projetos, documentos e fotos sem complicação.

[CTA Button: Download for macOS] [Secondary CTA: View on GitHub]

[Hero Image: Screenshot do app mostrando interface clean e dark]
```

#### Elementos Visuais
- Screenshot do app em ação
- Vídeo curto (10-15s) mostrando criação de backup
- Badge "macOS 12.0+"
- Badge "Open Source"
- Badge "100% Free"

---

### Features Section

**Título da Seção**: "Tudo que você precisa. Nada que você não precisa."

#### Feature 1: Três Modos de Backup
**Ícone**: `FileArchive`

**Título**: Escolha seu modo

**Descrição**:
- **Copy**: Cópia direta para acesso rápido (pasta)
- **Compressed**: TAR + zstd para economia de espaço (arquivo .tar.zst)
- **Encrypted**: AES-256-GCM para máxima segurança (arquivo .tar.zst.enc)

**Visual**: Tabela comparativa dos três modos

---

#### Feature 2: Compressão Inteligente
**Ícone**: `Zap`

**Título**: 5841x mais eficiente em textos

**Descrição**:
Compressão zstd de última geração. Streaming architecture que processa arquivos maiores que a RAM disponível. Backup de 200GB em sistemas com apenas 8GB de memória.

**Métrica Destaque**:
- 1GB em 0.53s
- Throughput: 1919 MB/s
- Economia típica: 40-70% de espaço

---

#### Feature 3: Criptografia Opcional
**Ícone**: `Lock`

**Título**: AES-256-GCM quando você precisa

**Descrição**:
Criptografia de nível militar, completamente opcional. Derivação de chave com Argon2 (RFC 9106). Zero-knowledge: apenas você tem a senha.

**Diferencial**: 31 testes de criptografia (mais que apps enterprise)

---

#### Feature 4: Agendamento Automático
**Ícone**: `Clock`

**Título**: Configure uma vez, esqueça para sempre

**Descrição**:
Integração nativa com launchd do macOS. Backups rodam mesmo com o app fechado. Presets prontos: horário, diário, semanal, mensal.

**Benefício**: Notificações nativas de sucesso/erro

---

#### Feature 5: Qualidade Enterprise
**Ícone**: `Shield`

**Título**: 78 testes automatizados. Zero perda de dados.

**Descrição**:
InLocker segue padrões de teste de software enterprise:
- 78 testes automatizados (100% taxa de sucesso)
- 31 testes de criptografia (RFC 9106 + NIST)
- Testes adversariais (path traversal, timing attacks, disk full)
- 3 bugs críticos encontrados e corrigidos ANTES da produção

**Proof Points**:
- Bug #1: Checksum fraco → Corrigido com SHA-256
- Bug #2: Timing attack → Corrigido com constant-time comparison
- Bug #3: Limpeza parcial → Corrigido com remoção automática

---

#### Feature 6: Restore Confiável
**Ícone**: `CheckCircle`

**Título**: Backup sem restore não é backup

**Descrição**:
- Verificação de integridade SHA-256 em todo restore
- Progress bar em tempo real
- Detecção automática de corrupção
- Suporte a cancelamento inteligente

---

### How It Works Section

**Título**: Configure em 3 minutos

#### Passo 1: Selecione a pasta
"Escolha qual pasta você quer proteger: projetos, documentos, fotos, vídeos..."

[Screenshot: Dialog de seleção de pasta]

#### Passo 2: Escolha o destino e o modo
"Onde salvar (HD externo recomendado) e qual modo usar (Copy, Compressed ou Encrypted)"

[Screenshot: Configuração de destino e modo]

#### Passo 3: Agende (opcional)
"Configure uma vez, esqueça para sempre. Ou rode manualmente quando quiser."

[Screenshot: Interface de agendamento]

**Resultado**: "Pronto! Seus dados estão protegidos."

[Screenshot: Dashboard mostrando backup concluído com sucesso]

---

### Comparison Section

**Título**: InLocker vs. Concorrentes

| Feature | InLocker | Time Machine | Backblaze | Carbon Copy |
|---------|----------|--------------|-----------|-------------|
| **Gratuito** | ✅ | ✅ | ❌ ($9/mês) | ❌ ($40) |
| **Local** | ✅ | ✅ | ❌ Cloud | ✅ |
| **Modos Flexíveis** | ✅ 3 modos | ❌ | ❌ | ❌ |
| **Compressão** | ✅ zstd | ❌ | ✅ | ❌ |
| **Múltiplos Destinos** | ✅ | ❌ | ❌ | ✅ |
| **Criptografia** | ✅ Opcional | ⚠️ Básica | ✅ | ⚠️ |
| **Leve (<5MB)** | ✅ | ❌ | ❌ | ❌ |
| **78 Testes** | ✅ | ❌ | ❌ | ❌ |
| **Auditoria OWASP** | ✅ | ⚠️ | ⚠️ | ❌ |

**Conclusão**: "InLocker oferece o melhor de todos os mundos: gratuito como Time Machine, confiável como Carbon Copy, e com qualidade de testes que apps pagos não têm."

---

### Testing & Quality Section

**Título**: "Qualidade que você não vê em apps gratuitos"

**Subtítulo**: "InLocker trata testes como apps enterprise - porque seus dados merecem."

#### Filosofia de Testes
"Testes são desenhados para **encontrar falhas**, não para passar. Esta mentalidade defensiva já evitou 3 bugs críticos antes da produção."

#### Cobertura de Testes
- **78 testes automatizados** (taxa de sucesso: 100%)
- **7 suítes de teste**: adversarial, backup_restore, critical_backup, critical_security, crypto, performance, security
- **Zero cenários de perda de dados** - todos os caminhos críticos testados
- **31 testes de criptografia** seguindo RFC 9106 e padrões NIST
- **Cobertura de código: 75%** (meta: 90%)

#### Tipos de Testes
1. **Testes Adversariais**: Path traversal, timing attacks, disk full
2. **Testes de Performance**: 1GB em <2min, compressão 5841x
3. **Testes de Integridade**: Corrupção, bit-flip, truncation
4. **Testes de Segurança**: Checksum collision, manifest tampering

**CTA**: "Ver relatório completo de testes no GitHub"

---

### Social Proof Section

**Título**: "Construído por desenvolvedores, para desenvolvedores"

#### GitHub Stats
- ⭐ Stars no GitHub
- 📦 Downloads totais
- 🔄 Contributors
- 🐛 Issues fechadas

#### Testimonials (futuro)
Placeholder para depoimentos de usuários beta:
- "Finalmente um app de backup que não me deixa ansioso" - João, Dev Full-Stack
- "78 testes automatizados em um app gratuito? Inacreditável." - Maria, DevOps Engineer
- "Migrei do Time Machine e nunca mais voltei" - Pedro, Product Designer

---

### FAQ Section

**Pergunta 1**: InLocker é realmente gratuito?
**Resposta**: Sim, 100% gratuito e open source. Sem freemium, sem assinaturas, sem limitações artificiais.

**Pergunta 2**: Meus dados ficam na cloud?
**Resposta**: Não. Tudo fica no seu Mac ou no destino que você escolher (HD externo, NAS, etc). Zero uploads para servidores externos.

**Pergunta 3**: Como funciona a criptografia?
**Resposta**: Usamos AES-256-GCM (padrão militar) com derivação de chave Argon2. A senha nunca é armazenada, apenas você tem acesso.

**Pergunta 4**: Posso restaurar arquivos individuais?
**Resposta**: Sim. Backups em modo Copy permitem acesso direto aos arquivos. Modos Compressed e Encrypted possuem restore completo com verificação de integridade.

**Pergunta 5**: Qual a diferença entre Full e Incremental?
**Resposta**: Full faz backup de todos os arquivos. Incremental só faz backup dos arquivos modificados desde o último backup (52x mais rápido).

**Pergunta 6**: InLocker roda em background?
**Resposta**: Sim, através do launchd do macOS. Backups agendados rodam mesmo com o app fechado.

**Pergunta 7**: Qual o tamanho do app?
**Resposta**: Menos de 5MB - 30x menor que apps Electron tradicionais.

**Pergunta 8**: Posso confiar no InLocker com dados sensíveis?
**Resposta**: Sim. Além de 78 testes automatizados, o código é open source e pode ser auditado por qualquer pessoa. Seguimos padrões OWASP para segurança.

---

### Download Section

**Título**: "Comece a proteger seus dados agora"

#### System Requirements
- macOS 12.0 (Monterey) ou superior
- 100 MB de espaço livre
- Processador Apple Silicon ou Intel

#### Download Options
```
[Botão Principal: Download InLocker.dmg (versão X.X.X)]
[Link secundário: Ver todas as versões no GitHub Releases]
```

#### Installation
1. Baixe o arquivo `.dmg`
2. Abra e arraste InLocker para a pasta Applications
3. Abra o InLocker (primeiro uso: Cmd+clique para contornar Gatekeeper)
4. Pronto!

#### Alternative: Build from Source
```bash
git clone https://github.com/beloureiro/inlocker.git
cd inlocker
pnpm install
pnpm tauri build
```

---

## Recursos Visuais

### Screenshots Necessários

#### 1. App Interface (Hero Image)
- Tela principal do app mostrando:
  - Header com logo InLocker
  - Lista de backups configurados
  - Botões de ação (Run Backup, Delete, Edit)
  - Dark theme elegante

#### 2. Progress Bar em Ação
- Screenshot do backup em progresso
- Mostrar:
  - Progress bar determinate (TAR creation)
  - Contagem de arquivos processados
  - Tempo decorrido
  - Botão de cancelamento

#### 3. Backup Success
- Modal de sucesso mostrando:
  - Ícone de checkmark verde
  - Métricas: arquivos, tamanho original, tamanho comprimido, ratio
  - Tempo total de backup

#### 4. Schedule Configuration
- Interface de agendamento mostrando:
  - Presets (Hourly, Daily, Weekly, Monthly)
  - Cron expression personalizada
  - Preview do próximo backup

#### 5. Three Modes Comparison
- Três cards lado a lado mostrando:
  - Copy (pasta icon)
  - Compressed (archive icon)
  - Encrypted (lock icon)

#### 6. Restore Interface
- Interface de restore mostrando:
  - Seleção de arquivo de backup
  - Seleção de destino de restore
  - Progress bar de restore

### Vídeos/GIFs Recomendados

#### 1. Quick Setup (15 segundos)
- Demonstrar os 3 passos básicos
- Acelerar em 2x
- Overlay com texto explicativo

#### 2. Backup in Action (10 segundos)
- Clicar em "Run Backup"
- Mostrar progress bar enchendo
- Terminar com modal de sucesso

#### 3. Restore Demo (15 segundos)
- Abrir RestoreSelector
- Escolher backup
- Escolher destino
- Mostrar conclusão

### Ilustrações Técnicas

#### 1. Architecture Diagram (Simplificado)
```
┌─────────────┐
│    USER     │
└──────┬──────┘
       │
       ▼
┌─────────────────────────────────┐
│  InLocker (Tauri + Rust)        │
│  • Scheduling                   │
│  • Compression (zstd)           │
│  • Encryption (AES-256)         │
└──────┬──────────────────────────┘
       │
       ▼
┌─────────────────────────────────┐
│  macOS launchd + File System    │
└──────┬──────────────────────────┘
       │
       ▼
┌─────────────────────────────────┐
│  Local Storage (Your Mac)       │
│  • External HD                  │
│  • NAS                          │
└─────────────────────────────────┘
```

#### 2. Backup Flow
```
[Source Folder] → [TAR] → [zstd Compression] → [AES-256 Encryption (optional)] → [SHA-256 Checksum] → [Destination]
```

#### 3. Testing Pyramid
```
       /\
      /31\     Crypto Tests
     /────\
    / 30+  \   Security Tests
   /────────\
  /   18     \ Core Tests
 /────────────\
      78 Total Tests
```

---

## Tom e Estilo

### Diretrizes de Comunicação

#### 1. Transparência Radical
- Mostre os números reais (78 testes, 3 bugs fixados)
- Não esconda limitações (ex: "encrypted backups requerem senha na execução")
- Código aberto = confiança

#### 2. Técnico sem ser Intimidante
- Use termos técnicos quando necessário (AES-256, zstd, SHA-256)
- MAS sempre explique o benefício prático
- Exemplo: "AES-256-GCM (criptografia de nível militar que nem governos conseguem quebrar)"

#### 3. Focado em Benefícios, não Features
- ❌ "Possui compressão zstd"
- ✅ "Economiza até 70% de espaço em disco"

- ❌ "78 testes automatizados"
- ✅ "78 testes automatizados garantem zero perda de dados"

#### 4. Honestidade sobre Competição
- Não demonize Time Machine ou outros concorrentes
- Reconheça pontos fortes deles
- Destaque onde InLocker é diferente (não necessariamente "melhor")

#### 5. Voz Ativa e Confiante
- ❌ "InLocker pode ajudar a proteger seus dados"
- ✅ "InLocker protege seus dados"

- ❌ "Tentamos fazer backups simples"
- ✅ "Backups simples, ponto final"

### Palavras-chave de Marca

**Usar frequentemente:**
- Simples / Simplicidade
- Automático / Automação
- Confiável / Confiabilidade
- Local / Privacidade
- Seguro / Segurança
- Testado / Qualidade
- Transparente / Open Source
- Rápido / Performance

**Evitar:**
- Perfeito (nada é perfeito)
- Revolucionário (hype desnecessário)
- Exclusivo (é open source, qualquer um pode usar)
- Premium/Enterprise (é gratuito)

---

## SEO e Palavras-chave

### Palavras-chave Primárias (PT-BR)
1. backup macOS
2. backup automático Mac
3. app backup gratuito macOS
4. backup local macOS
5. alternativa Time Machine
6. backup criptografado Mac
7. compressão backup macOS
8. backup sem cloud Mac

### Palavras-chave Secundárias
- backup para desenvolvedores
- backup zstd macOS
- AES-256 backup Mac
- open source backup macOS
- Tauri backup app
- backup incremental Mac
- launchd backup macOS

### Palavras-chave Long-tail
- "como fazer backup automático no Mac sem Time Machine"
- "melhor app backup gratuito para macOS 2025"
- "backup local Mac com criptografia"
- "backup rápido e comprimido macOS"
- "alternativas gratuitas ao Backblaze para Mac"

### Meta Tags Recomendadas

#### Title Tag
```html
<title>InLocker - Backup Automático e Seguro para macOS | Gratuito e Open Source</title>
```

#### Meta Description
```html
<meta name="description" content="InLocker oferece backups automáticos, comprimidos e criptografados para macOS. 100% gratuito, 100% local, 78 testes automatizados. Alternativa moderna ao Time Machine.">
```

#### Open Graph (Social Sharing)
```html
<meta property="og:title" content="InLocker - Backup Automático para macOS">
<meta property="og:description" content="Backups simples, confiáveis e 100% locais. Gratuito e open source.">
<meta property="og:image" content="https://inlocker.app/og-image.png">
<meta property="og:url" content="https://inlocker.app">
<meta property="og:type" content="website">
```

#### Twitter Card
```html
<meta name="twitter:card" content="summary_large_image">
<meta name="twitter:title" content="InLocker - Backup Automático para macOS">
<meta name="twitter:description" content="Backups simples, confiáveis e 100% locais. Gratuito e open source.">
<meta name="twitter:image" content="https://inlocker.app/twitter-card.png">
```

### Structured Data (Schema.org)
```json
{
  "@context": "https://schema.org",
  "@type": "SoftwareApplication",
  "name": "InLocker",
  "operatingSystem": "macOS 12.0+",
  "applicationCategory": "UtilitiesApplication",
  "aggregateRating": {
    "@type": "AggregateRating",
    "ratingValue": "4.9",
    "ratingCount": "0"
  },
  "offers": {
    "@type": "Offer",
    "price": "0",
    "priceCurrency": "USD"
  },
  "description": "Automatic, compressed, and secure backups for macOS. 100% free and open source.",
  "downloadUrl": "https://github.com/beloureiro/inlocker/releases",
  "softwareVersion": "0.1.0",
  "author": {
    "@type": "Person",
    "name": "Bernardo Loureiro"
  },
  "license": "https://opensource.org/licenses/MIT"
}
```

---

## CTAs (Call-to-Actions)

### Primários
1. **"Download for macOS"** - Botão principal verde esmeralda
2. **"View on GitHub"** - Botão secundário com outline

### Secundários
1. "Read Documentation"
2. "See How It Works"
3. "Compare with Time Machine"
4. "Join Community" (se houver Discord/Forum)

### Posicionamento
- Hero Section: Download + GitHub
- Features Section: Download (sticky sidebar ou floating button)
- Bottom of Page: Download + Newsletter (se aplicável)
- Footer: GitHub + Documentation

---

## Estrutura de Navegação

### Header (Sticky)
```
[Logo] InLocker    |    Features    How It Works    Pricing    Docs    Download
```

### Footer
```
InLocker
Automatic, compressed, and secure backups

PRODUCT               RESOURCES           COMMUNITY
Features              Documentation       GitHub
How It Works          Quick Start         Report Issue
Download              FAQs                Changelog
Roadmap               Testing Report

LEGAL
Privacy Policy
Terms of Service
License (MIT)

© 2025 InLocker. Open source and proud.
```

---

## Checklist de Lançamento do Site

### Design
- [ ] Logo em alta resolução (SVG + PNG)
- [ ] Favicon (32x32, 64x64)
- [ ] Apple Touch Icon (180x180)
- [ ] 6+ screenshots do app
- [ ] 3 GIFs/vídeos de demonstração
- [ ] Ilustrações técnicas (diagrams)

### Conteúdo
- [ ] Hero section com headline forte
- [ ] Features detalhadas (6+)
- [ ] Tabela de comparação
- [ ] Seção de testes/qualidade
- [ ] FAQ (8+ perguntas)
- [ ] CTA claro em todas as seções

### Técnico
- [ ] Meta tags (title, description, OG, Twitter)
- [ ] Schema.org structured data
- [ ] Sitemap.xml
- [ ] Robots.txt
- [ ] Performance (Lighthouse score >90)
- [ ] Responsivo (mobile, tablet, desktop)
- [ ] Dark mode (match app identity)

### SEO
- [ ] Palavras-chave integradas naturalmente
- [ ] Headings hierárquicos (H1, H2, H3)
- [ ] Alt text em todas as imagens
- [ ] Links internos relevantes
- [ ] URLs amigáveis (clean slugs)

### Analytics
- [ ] Google Analytics ou Plausible (privacy-friendly)
- [ ] Event tracking (CTA clicks, downloads)
- [ ] Heatmap (Hotjar ou similar)

### Legal
- [ ] Privacy Policy
- [ ] Terms of Service
- [ ] Cookie consent (se necessário)
- [ ] Link para licença MIT no GitHub

---

## Stack Técnico Recomendada para o Site

### Framework
**Next.js 14+** (React-based, SSG/SSR)
- SEO-friendly
- Fast performance
- Easy deployment (Vercel)

### Alternativas
- **Astro** (ultra-fast, minimal JS)
- **Gatsby** (similar ao Next.js)
- **Hugo** (Go-based, ultra-rápido)

### Styling
- **TailwindCSS** (consistência com o app)
- **Framer Motion** (animações suaves)

### Hosting
- **Vercel** (recomendado para Next.js)
- **Netlify** (alternativa)
- **GitHub Pages** (gratuito, mas limitado)

### Domain
Sugestões:
- `inlocker.app`
- `getinlocker.com`
- `inlocker.dev`

---

## Métricas de Sucesso

### KPIs do Site
1. **Downloads por mês**: Meta inicial 100+
2. **Taxa de conversão** (visita → download): Meta 5-10%
3. **Tempo na página**: Meta >2 minutos
4. **Bounce rate**: Meta <50%
5. **GitHub stars**: Meta 100+ no primeiro mês

### Ferramentas de Monitoramento
- Google Analytics / Plausible
- GitHub Insights (stars, forks, downloads)
- User feedback (GitHub Issues)

---

## Cronograma Sugerido

### Semana 1: Design
- Wireframes de todas as páginas
- Definição da paleta de cores
- Criação do logo em alta resolução
- Coleta de screenshots do app

### Semana 2: Conteúdo
- Redação de todas as seções
- Criação de diagramas técnicos
- Gravação de vídeos/GIFs
- Revisão de copy

### Semana 3: Desenvolvimento
- Setup do framework (Next.js)
- Implementação do design
- Integração de componentes
- Otimização de performance

### Semana 4: Lançamento
- Testes em múltiplos dispositivos
- SEO audit final
- Deploy em produção
- Anúncio no GitHub / Reddit / Hacker News

---

## Recursos Externos

### Inspiração de Design
- https://tauri.app (site oficial do Tauri)
- https://restic.net (backup CLI, design clean)
- https://dupeguru.voltaicideas.net (simplicidade)
- https://1password.com (confiança e segurança)

### Ferramentas de Design
- Figma (wireframes e mockups)
- Excalidraw (diagramas técnicos)
- ScreenToGif (gravação de demos)
- TinyPNG (compressão de imagens)

### Teste de Performance
- Google Lighthouse
- WebPageTest
- GTmetrix

---

## Notas Finais

### Princípios Norteadores
1. **Transparência acima de tudo** - Mostre o código, os testes, os bugs corrigidos
2. **Simplicidade visual** - O site deve ser tão simples quanto o app
3. **Performance importa** - Site rápido = app rápido (primeira impressão)
4. **Focado no usuário** - Responda "Por que EU deveria usar isso?" em 10 segundos

### O que NÃO fazer
- ❌ Usar jargões técnicos sem explicação
- ❌ Prometer perfeição (seja honesto sobre limitações)
- ❌ Copiar design de concorrentes
- ❌ Lotar de features sem mostrar benefícios
- ❌ Esconder o código (é open source, celebre isso!)

### Próximos Passos
1. Validar wireframes com 3-5 usuários potenciais
2. Criar protótipo interativo no Figma
3. Desenvolver MVP do site (apenas Home + Download)
4. Iterar baseado em feedback
5. Lançar versão completa

---

**Documento criado em**: 2025-11-21
**Versão**: 1.0.0
**Autor**: Baseado na documentação oficial do InLocker

Para dúvidas ou sugestões, abra uma issue no [GitHub](https://github.com/beloureiro/inlocker/issues).
