#!/bin/bash

echo "🧹 Limpando instâncias antigas do InLocker.app"
echo "=============================================="
echo ""

# 1. Matar qualquer processo do InLocker rodando
echo "1. Encerrando processos do InLocker..."
pkill -9 -i inlocker 2>/dev/null
echo "   ✓ Processos encerrados"
echo ""

# 2. Remover da pasta Applications
echo "2. Removendo do /Applications..."
if [ -d "/Applications/InLocker.app" ]; then
    rm -rf "/Applications/InLocker.app"
    echo "   ✓ Removido: /Applications/InLocker.app"
else
    echo "   • Não encontrado em /Applications"
fi
echo ""

# 3. Manter apenas a versao de desenvolvimento (escolha debug OU release)
echo "3. Limpando builds antigos..."
echo "   Escolha qual manter: [d]ebug ou [r]elease? (padrao: debug)"
read -t 5 choice || choice="d"

if [ "$choice" = "r" ]; then
    # Manter release, remover debug
    if [ -d "./src-tauri/target/debug" ]; then
        rm -rf "./src-tauri/target/debug"
        echo "   ✓ Removido: debug build"
    fi
    echo "   • Mantido: release build"
else
    # Manter debug, remover release
    if [ -d "./src-tauri/target/release" ]; then
        rm -rf "./src-tauri/target/release"
        echo "   ✓ Removido: release build"
    fi
    echo "   • Mantido: debug build"
fi
echo ""

# 4. Verificar o que sobrou
echo "4. Verificando instâncias restantes..."
REMAINING=$(mdfind "kMDItemFSName == 'InLocker.app'" 2>/dev/null | grep -v ".Trash" | wc -l)
echo "   • Instâncias encontradas: $REMAINING"
echo ""

if [ "$REMAINING" -eq 1 ]; then
    echo "✅ Limpeza concluída! Apenas 1 instância (debug) permanece."
    echo ""
    mdfind "kMDItemFSName == 'InLocker.app'" 2>/dev/null | grep -v ".Trash"
else
    echo "⚠️  Ainda existem $REMAINING instâncias:"
    mdfind "kMDItemFSName == 'InLocker.app'" 2>/dev/null | grep -v ".Trash"
fi
echo ""
echo "=============================================="