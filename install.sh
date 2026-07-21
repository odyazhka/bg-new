#!/bin/bash
BIN="bg"
echo "--- Установка управления яркостью (AMD/Intel) ---"

if [ ! -f "./$BIN" ]; then
    echo "❌ Ошибка: Файл '$BIN' не найден в этой папке!"
    ls -l
    exit 1
fi

if [ -n "$SUDO_USER" ]; then
    REAL_USER="$SUDO_USER"; REAL_HOME="/home/$SUDO_USER"
else
    REAL_USER="$USER"; REAL_HOME="$HOME"
fi

echo "   Настоящий пользователь: $REAL_USER"
echo "   Домашняя директория:    $REAL_HOME"

echo "[0/3] Создаю директорию $REAL_HOME/.local/ ..."
mkdir -p "$REAL_HOME/.local"
chown "$REAL_USER:$REAL_USER" "$REAL_HOME/.local" 2>/dev/null || true

echo "[1/3] Копирую бинарник в /usr/local/bin/brightness..."
chmod +x "./$BIN"
sudo cp "./$BIN" /usr/local/bin/brightness
sudo chmod +x /usr/local/bin/brightness

echo "[2/3] Создаю udev-правило (AMD + Intel)..."
echo 'ACTION=="add", SUBSYSTEM=="backlight", KERNEL=="amdgpu_bl*", RUN+="/bin/chmod 666 /sys/class/backlight/%k/brightness"
ACTION=="add", SUBSYSTEM=="backlight", KERNEL=="intel_backlight", RUN+="/bin/chmod 666 /sys/class/backlight/%k/brightness"' | sudo tee /etc/udev/rules.d/99-backlight.rules > /dev/null

echo "[3/3] Применяю настройки..."
sudo udevadm control --reload-rules
sudo udevadm trigger

for dev in amdgpu_bl0 amdgpu_bl1 intel_backlight; do
    if [ -d "/sys/class/backlight/$dev" ]; then
        sudo chmod 666 "/sys/class/backlight/$dev/brightness"
    fi
done

echo ""
echo "--- ✅ ВСЁ ГОТОВО! ---"
echo "Яркость сохраняется в: $REAL_HOME/.local/bg"
echo "Запускать (например, из polybar click-left): brightness"
