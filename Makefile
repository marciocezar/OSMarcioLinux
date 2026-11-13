# =============================================================================
#  OSMarcio Linux — Makefile central
#  `make help` lista todos os alvos. Alvos ainda não implementados falham com
#  erro claro (código 2) apontando a fase correspondente em PROGRESS.md.
#  Compatível com GNU Make 3.81 (padrão do macOS).
# =============================================================================

SHELL   := /bin/bash
VERSION := $(shell tr -d '[:space:]' < VERSION)
PROFILE ?= desktop
UNAME_S := $(shell uname -s)

IN_BUILDER := scripts/build/in-builder.sh
NOT_YET    := scripts/lib/not-yet.sh
PROFILE_CAP := $(shell echo $(PROFILE) | awk '{print toupper(substr($$0,1,1)) substr($$0,2)}')
UTM_IMG    ?= $(wildcard images/utm/OSMarcioLinux-UTM-AArch64-$(PROFILE_CAP)-v$(VERSION).img)

.DEFAULT_GOAL := help
.PHONY: help version doctor doctor-mac doctor-windows doctor-wsl \
        setup-mac setup-windows setup-wsl builder \
        apps-build apps-test apps-lint test-apps debs \
        browser-build browser-test terminal-build terminal-test terminal-deb \
        editor-build files-build settings-build software-center-build \
        distro-build rootfs-arm64 rootfs-x86_64 image-minimal image-desktop image-developer \
        build-arm64 build-x86_64 iso-x86_64 image-utm image-rockpi4a verify-images \
        run-utm run-qemu-arm64 run-qemu-x86_64 run-virtualbox browser-bundle info-deb \
        browser-run browser-package browser-version browser-check-updates browser-icons \
        browser-install browser-release test-browser-hotfix \
        test-boot test-desktop test test-network test-browser test-install test-utm test-virtualbox test-regression \
        docs docs-check diagrams release checksums release-verify clean

## ---------------------------------------------------------------- Ajuda
help: ## Lista os comandos disponíveis
	@echo "OSMarcio Linux $(VERSION) — alvos do Makefile  (PROFILE=$(PROFILE))"
	@awk 'BEGIN{FS=":.*## "} /^## -+ /{sub(/^## -+ /,""); printf "\n\033[1m%s\033[0m\n",$$0} /^[a-zA-Z0-9_-]+:.*## /{printf "  \033[36m%-24s\033[0m %s\n",$$1,$$2}' $(MAKEFILE_LIST)
	@echo; echo "Legenda: [planejado] = ainda não implementado (falha com erro claro)."

version: ## Mostra a versão da distribuição (não é a versão do kernel)
	@echo "OSMarcio Linux $(VERSION)"

## ---------------------------------------------------------------- Diagnóstico e setup
doctor: ## Diagnóstico automático do host atual
ifeq ($(UNAME_S),Darwin)
	@scripts/setup/setup-macos.sh --check
else
	@if grep -qi microsoft /proc/version 2>/dev/null; then scripts/setup/setup-wsl.sh --check; else echo "Host Linux nativo: use 'make doctor-wsl' (mesmos requisitos)"; scripts/setup/setup-wsl.sh --check; fi
endif

doctor-mac: ## Diagnóstico do macOS Apple Silicon
	@scripts/setup/setup-macos.sh --check
setup-mac: ## Prepara o macOS (pergunta antes de instalar qualquer coisa)
	@scripts/setup/setup-macos.sh
doctor-wsl: ## Diagnóstico do WSL2 (rodar dentro do WSL2)
	@scripts/setup/setup-wsl.sh --check
setup-wsl: ## Prepara o WSL2 (pergunta antes de instalar qualquer coisa)
	@scripts/setup/setup-wsl.sh
doctor-windows: ## Diagnóstico do Windows (via PowerShell)
	@command -v powershell.exe >/dev/null && powershell.exe -ExecutionPolicy Bypass -File scripts/setup/setup-windows.ps1 -Check || { echo "Rode no PowerShell do Windows: .\\scripts\\setup\\setup-windows.ps1 -Check"; exit 2; }
setup-windows: ## Prepara o Windows (via PowerShell)
	@command -v powershell.exe >/dev/null && powershell.exe -ExecutionPolicy Bypass -File scripts/setup/setup-windows.ps1 || { echo "Rode no PowerShell do Windows: .\\scripts\\setup\\setup-windows.ps1"; exit 2; }
builder: ## (Re)constrói o container de build Debian 13
	@scripts/build/builder-image.sh

## ---------------------------------------------------------------- Aplicativos Rust
apps-build: ## Compila todos os apps (Linux, no container)
	@$(IN_BUILDER) cargo build --workspace --locked
apps-lint: ## rustfmt + clippy (-D warnings)
	@$(IN_BUILDER) bash -c "cargo fmt --all --check && cargo clippy --workspace --all-targets --locked -- -D warnings"
apps-test: ## Testes unitários + teste de GUI do terminal (Xvfb)
	@$(IN_BUILDER) bash -c "cargo test --workspace --locked && cargo build -p osmarcio-terminal --locked && GTK_A11Y=none scripts/test/test-terminal-gui.sh"
test-apps: apps-test ## Alias de apps-test
debs: terminal-deb info-deb ## Gera os .deb do Terminal e do osmarcio-info (Browser: make browser-package)
info-deb: ## Gera out/debs/osmarcio-info_<versão>_<arch>.deb
	@$(IN_BUILDER) scripts/build/deb.sh osmarcio-info
browser-bundle: ## OSMarcio Browser (Rust + CEF) em out/browser-bundle-<arch>-v<versão>/
	@$(IN_BUILDER) scripts/build/browser-bundle.sh /work/out/browser-bundle-$(shell uname -m | sed 's/aarch64/arm64/;s/x86_64/amd64/')-v$(VERSION) release
terminal-build: ## Compila o OSMarcio Terminal
	@$(IN_BUILDER) cargo build -p osmarcio-terminal --locked
terminal-test: ## Testes do OSMarcio Terminal (unitários + GUI)
	@$(IN_BUILDER) bash -c "cargo test -p osmarcio-terminal -p osmarcio-core --locked && cargo build -p osmarcio-terminal --locked && GTK_A11Y=none scripts/test/test-terminal-gui.sh"
terminal-deb: ## Gera out/debs/osmarcio-terminal_<versão>_<arch>.deb
	@$(IN_BUILDER) scripts/build/deb.sh osmarcio-terminal
BROWSER_ARCH := $(shell uname -m | sed 's/aarch64/arm64/;s/x86_64/amd64/')
BROWSER_BUNDLE := out/browser-bundle-$(BROWSER_ARCH)-v$(VERSION)
BROWSER_DEB := out/debs/osmarcio-browser_$(VERSION)_$(BROWSER_ARCH).deb
browser-build: ## OSMarcio Browser: compila (release) e monta o bundle com o runtime CEF
	@$(IN_BUILDER) scripts/build/browser-bundle.sh /work/$(BROWSER_BUNDLE) release
browser-icons: ## Gera os PNGs 16…256 do ícone a partir do SVG original
	@$(IN_BUILDER) scripts/build/browser-icons.sh
browser-package: ## Gera out/debs/osmarcio-browser_<versão>_<arch>.deb (não sobrescreve)
	@$(IN_BUILDER) scripts/build/browser-deb.sh
browser-version: ## Versões reais (Browser, Chromium, CEF, cef-rs) do bundle compilado
	@test -x $(BROWSER_BUNDLE)/osmarcio-browser || { echo "bundle ausente: rode 'make browser-build'"; exit 1; }
	@$(IN_BUILDER) /work/$(BROWSER_BUNDLE)/osmarcio-browser --version
browser-check-updates: ## Consulta as fontes de atualização (no container; no sistema: osmarcio-browser --check-updates)
	@test -x $(BROWSER_BUNDLE)/osmarcio-browser || { echo "bundle ausente: rode 'make browser-build'"; exit 1; }
	@$(IN_BUILDER) /work/$(BROWSER_BUNDLE)/osmarcio-browser --check-updates
browser-run: ## Abre o bundle (somente num Linux com sessão gráfica; no Mac use a VM)
	@if [ "$(UNAME_S)" != Linux ] || [ -z "$$DISPLAY$$WAYLAND_DISPLAY" ]; then echo "browser-run precisa de Linux com sessão gráfica. No Mac: 'make run-utm' ou 'make run-qemu-arm64' e, na VM, 'osmarcio-browser'."; exit 2; fi
	@$(BROWSER_BUNDLE)/osmarcio-browser
browser-install: ## Instala o .deb com apt (somente em Debian/OSMarcio; pede confirmação)
	@test -f $(BROWSER_DEB) || { echo "pacote ausente: rode 'make browser-package'"; exit 1; }
	@command -v apt-get >/dev/null || { echo "browser-install é para Debian/OSMarcio Linux. Copie $(BROWSER_DEB) para a VM e rode: sudo apt install ./$$(basename $(BROWSER_DEB))"; exit 2; }
	@read -p "Instalar $(BROWSER_DEB) com 'sudo apt install'? [s/N] " a; [ "$$a" = s ] || [ "$$a" = S ] || { echo "cancelado"; exit 1; }
	@sudo apt install -y ./$(BROWSER_DEB)
browser-release: ## Copia o .deb + SHA-256 para images/releases/ (não sobrescreve)
	@test -f $(BROWSER_DEB) || $(MAKE) --no-print-directory browser-package
	@scripts/release/browser-release.sh $(BROWSER_DEB)
browser-test: ## Testes do browser: unitários (core) + hotfix na VM (QEMU/HVF, sessão gráfica real)
	@$(IN_BUILDER) cargo test -p osmarcio-core --locked
	@$(MAKE) --no-print-directory test-browser-hotfix
test-browser-hotfix: ## Fechamento, ícone, Sobre e atualizações dentro do OSMarcio Linux (QEMU/HVF)
	@test -n "$(UTM_IMG)" || { echo "imagem não encontrada; rode 'make image-utm'"; exit 1; }
	@scripts/test/test-browser-hotfix-qemu.sh "$(UTM_IMG)"
editor-build: ## [planejado] OSMarcio Editor — Fase 8
	@$(NOT_YET) $@ "Fase 8"
files-build: ## [planejado] OSMarcio Files — Fase 9
	@$(NOT_YET) $@ "Fase 9"
settings-build: ## [planejado] OSMarcio Settings — Fase 13
	@$(NOT_YET) $@ "Fase 13"
software-center-build: ## [planejado] OSMarcio Software Center — Fase 11
	@$(NOT_YET) $@ "Fase 11"

## ---------------------------------------------------------------- Distribuição e imagens
rootfs-arm64: ## Gera somente o rootfs ARM64 (out/rootfs-arm64-<perfil>.tar)
	@$(IN_BUILDER) scripts/build/rootfs.sh arm64 $(PROFILE) out/rootfs-arm64-$(PROFILE).tar
rootfs-x86_64: ## Gera somente o rootfs x86_64 (emulado no Mac; nativo no WSL2)
	@$(IN_BUILDER) scripts/build/rootfs.sh amd64 $(PROFILE) out/rootfs-amd64-$(PROFILE).tar
image-utm: ## Gera a IMG ARM64 UEFI para UTM (PROFILE=desktop padrão; minimal = só console)
	@if [ "$(PROFILE)" != minimal ]; then for d in terminal-deb:osmarcio-terminal info-deb:osmarcio-info browser-package:osmarcio-browser; do \
	  f=out/debs/$${d#*:}_$(VERSION)_$(BROWSER_ARCH).deb; if [ -f $$f ]; then echo "usando $$f (já existe)"; else $(MAKE) --no-print-directory $${d%%:*} || exit 1; fi; done; fi
	@scripts/image/build-disk-image.sh arm64 $(PROFILE) utm
build-arm64: image-utm ## Alias: imagem ARM64 completa
image-minimal: ## Imagem UTM do perfil minimal
	@$(MAKE) --no-print-directory image-utm PROFILE=minimal
image-desktop: ## [planejado] Perfil desktop — Fase 6
	@$(NOT_YET) $@ "Fase 6" "perfil desktop (labwc + apps OSMarcio) ainda não definido"
image-developer: ## [planejado] Perfil developer — Fase 12
	@$(NOT_YET) $@ "Fase 12"
distro-build: image-utm ## Alias: constrói a distribuição para o host atual
build-x86_64: ## [planejado] Imagem x86_64 — Fase 5
	@$(NOT_YET) $@ "Fase 5"
iso-x86_64: ## [planejado] ISO instalável x86_64 para VirtualBox — Fases 5/15
	@$(NOT_YET) $@ "Fases 5 e 15" "requer ISO híbrida UEFI + instalador (ADR 0004)"
image-rockpi4a: ## [planejado] Imagem RK3399 — Fase 14 (nunca derivada da imagem UTM)
	@$(NOT_YET) $@ "Fase 14" "requer U-Boot + TF-A + DTB do RK3399 validados"
verify-images: ## Verifica SHA-256 de todas as imagens em images/
	@scripts/release/verify-images.sh
checksums: verify-images ## Alias de verify-images (checksums são gerados no build)

## ---------------------------------------------------------------- Execução
run-qemu-arm64: ## Executa a IMG ARM64 no QEMU com janela (disco persistente em out/vm/)
	@test -n "$(UTM_IMG)" || { echo "imagem não encontrada; rode 'make image-utm'"; exit 1; }
	@scripts/run/qemu-arm64.sh "$(UTM_IMG)"
run-utm: ## Cria (1ª vez) e inicia a VM no app UTM; nunca sobrescreve discos existentes
	@test -n "$(UTM_IMG)" || { echo "imagem não encontrada; rode 'make image-utm'"; exit 1; }
	@scripts/run/utm.sh "$(UTM_IMG)"
run-qemu-x86_64: ## [planejado] — Fase 5
	@$(NOT_YET) $@ "Fase 5"
run-virtualbox: ## [planejado] VM VirtualBox — Fase 5 (script em scripts/windows/)
	@$(NOT_YET) $@ "Fase 5" "o script PowerShell existe, mas ainda não há ISO para anexar"

## ---------------------------------------------------------------- Testes
test: apps-lint apps-test test-utm ## Todos os testes disponíveis
test-utm: ## Boot real da IMG ARM64 (QEMU/HVF, UEFI, 2 boots, autoteste interno)
	@test -n "$(UTM_IMG)" || { echo "imagem não encontrada; rode 'make image-utm'"; exit 1; }
	@scripts/test/test-boot-qemu-arm64.sh "$(UTM_IMG)"
test-boot: test-utm ## Alias: firmware → GRUB → kernel → systemd → login (+ LightDM no desktop)
test-desktop: ## Login gráfico real + XFCE + Terminal + Browser (QEMU/HVF, captura de tela)
	@test -n "$(UTM_IMG)" || { echo "imagem não encontrada; rode 'make image-utm'"; exit 1; }
	@scripts/test/test-desktop-qemu-arm64.sh "$(UTM_IMG)"
test-network: test-utm ## Rede é verificada dentro do autoteste de boot (DHCP/DNS/HTTPS/APT)
test-regression: test ## Alias de test
test-browser: browser-test ## Alias de browser-test
test-install: ## [planejado] — Fase 15
	@$(NOT_YET) $@ "Fase 15"
test-virtualbox: ## [planejado] — Fase 5
	@$(NOT_YET) $@ "Fase 5"

## ---------------------------------------------------------------- Documentação
docs: docs-check ## Valida a documentação
docs-check: ## Verifica links internos dos .md e XML dos diagramas
	@python3 scripts/test/docs-check.py
diagrams: ## Valida os .drawio e exporta PNG (requer draw.io desktop)
	@scripts/release/diagrams.sh

## ---------------------------------------------------------------- Releases
release: ## [planejado] — Fase 18
	@$(NOT_YET) $@ "Fase 18" "nenhuma release estável sem ISO/IMG validadas"
release-verify: verify-images ## Verifica checksums dos artefatos

clean: ## Remove artefatos de compilação/teste (NUNCA apaga out/vm nem images/)
	rm -rf out/target-linux-* out/test-* out/debs target
