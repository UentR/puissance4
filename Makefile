# Makefile pour le projet Chess System
# Automatise la compilation, l'exécution et le profilage

# Variables
CARGO_CMD = cargo
TARGET_BIN = ./target/release/arbitre
PACKAGE = arbitre

.PHONY: all build profile clean check run

# Cible par défaut (lancée si tu tapes juste `make`)
all: build

# 1. Compilation optimisée (ne recompile que si un fichier a été modifié)
build:
	@echo "==> Compilation du projet en mode Release..."
	$(CARGO_CMD) build --release -p $(PACKAGE)

# 2. Exécution classique du projet
run: build
	@echo "==> Exécution de l'Arbitre..."
	$(TARGET_BIN)

# 3. Profilage automatique avec samply
profile: build
	@echo "==> Lancement du profilage avec samply sur $(TARGET_BIN)..."
	@if command -v samply > /dev/null; then \
		samply record $(TARGET_BIN); \
	else \
		echo "Erreur : 'samply' n'est pas installé. Installez-le avec : cargo install samply"; \
		exit 1; \
	fi

# 4. Vérification très rapide (idéal pendant que tu codes)
check:
	@echo "==> Vérification du code avec cargo check..."
	$(CARGO_CMD) check

# 5. Nettoyage complet
clean:
	@echo "==> Nettoyage du cache de compilation..."
	$(CARGO_CMD) clean