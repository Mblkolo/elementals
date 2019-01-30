import * as wasm from "../pkg";
import nipplejs from "nipplejs";

const game = new wasm.Game();
console.log(game.get_state());

const CELL_SIZE = 20; // px

const canvas = document.getElementById("game-canvas");
canvas.height = window.innerHeight;
canvas.width = window.innerWidth;

const debugEl = document.getElementById("debug");
function debug(info) {
    debugEl.innerHTML = JSON.stringify(info, 0, 2);
}

var isTouchDevice = "ontouchstart" in document.documentElement;
if (isTouchDevice) {
    // screen joysticks

    const moveJoystick = nipplejs.create({
        zone: document.getElementById("left-joystick-zone"),
        color: "blue"
    });

    moveJoystick.on("end move", (event, data) => {
        if (event.type === "end") {
            game.set_player_direction(0, 0);
            return;
        }

        if (data.direction) {
            let x = data.instance.frontPosition.x / 50;
            let y = data.instance.frontPosition.y / 50;

            game.set_player_direction(x, y);
        }
    });

    const fireJoystick = nipplejs.create({
        zone: document.getElementById("right-joystick-zone"),
        color: "red"
    });

    const aimEl = document.getElementById("player-aim");

    fireJoystick.on("start end move", (event, data) => {
        switch (event.type) {
            case "start":
                game.set_shooting(true);
                break;
            case "end":
                game.set_shooting(false);
                break;
            case "move":
                if (data.direction) {
                    const frontPosition = data.instance.frontPosition;

                    const player = JSON.parse(game.get_player_pos());
                    if (player == null) {
                        break;
                    }

                    const aimPosX = player.x * CELL_SIZE + frontPosition.x;
                    const aimPosY = player.y * CELL_SIZE + frontPosition.y;

                    aimEl.style.left = `${aimPosX}px`;
                    aimEl.style.top = `${aimPosY}px`;

                    game.set_shoot_point(aimPosX / CELL_SIZE, aimPosY / CELL_SIZE);
                }
                break;

            default:
                break;
        }
    });
}

// mouse

canvas.addEventListener("mousemove", event => {
    game.set_shoot_point(event.offsetX / CELL_SIZE, event.offsetY / CELL_SIZE);
});

document.addEventListener("mousedown", event => {
    game.set_shooting(true);
});

document.addEventListener("mouseup", event => {
    game.set_shooting(false);
});

// keyboard

const player_speed = { x: 0, y: 0 };

document.addEventListener("keydown", event => {
    if (event.code == "KeyA") player_speed.x = -1;
    if (event.code == "KeyD") player_speed.x = +1;

    if (event.code == "KeyS") player_speed.y = +1;
    if (event.code == "KeyW") player_speed.y = -1;

    game.set_player_direction(player_speed.x, player_speed.y);
});

document.addEventListener("keyup", event => {
    if (event.code == "KeyA" && player_speed.x == -1) player_speed.x = 0;
    if (event.code == "KeyD" && player_speed.x == +1) player_speed.x = 0;

    if (event.code == "KeyS" && player_speed.y == +1) player_speed.y = 0;
    if (event.code == "KeyW" && player_speed.y == -1) player_speed.y = 0;

    game.set_player_direction(player_speed.x, player_speed.y);
});

let shoot_force = 1;
game.set_shoot_force(shoot_force);

document.addEventListener("keypress", event => {
    if (event.code == "Space") {
        shoot_force *= -1;
        game.set_shoot_force(shoot_force);
    }
});

const ctx = canvas.getContext("2d");

function draw(state) {
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    ctx.strokeRect(0, 0, 50 * CELL_SIZE, 40 * CELL_SIZE);

    // ctx.strokeStyle = "#444";
    // ctx.fillText(world.get_scope(), 950, 50);

    ctx.strokeStyle = "#000";

    for (let i = 0; i < state.enemies.length; ++i) {
        const enemy = state.enemies[i];

        ctx.fillStyle = "#000";
        if (enemy.is_white) {
            ctx.fillStyle = "#fff";
        }

        ctx.beginPath();
        ctx.arc(enemy.x * CELL_SIZE, enemy.y * CELL_SIZE, CELL_SIZE * enemy.radius, 0, 2 * Math.PI);
        ctx.fill();
        ctx.stroke();
    }

    ctx.strokeStyle = "#aaa";
    for (let i = 0; i < state.shots.length; ++i) {
        const shot = state.shots[i];
        ctx.beginPath();
        ctx.moveTo(shot.from_x * CELL_SIZE, shot.from_y * CELL_SIZE);
        ctx.lineTo(shot.to_x * CELL_SIZE, shot.to_y * CELL_SIZE);
        ctx.stroke();
    }

    ctx.strokeStyle = "#000";

    if (shoot_force > 0) ctx.fillStyle = "#fff";
    else ctx.fillStyle = "#000";

    const player = state.player;
    if (player != null) {
        ctx.beginPath();
        ctx.arc(player.x * CELL_SIZE, player.y * CELL_SIZE, CELL_SIZE * player.radius, 0, 2 * Math.PI);
        ctx.fill();
        ctx.stroke();
    }
}

setInterval(() => {
    const state = JSON.parse(game.get_state());
    if (state.player == null) {
        return;
    }

    draw(state);
    game.step();
}, 20);
