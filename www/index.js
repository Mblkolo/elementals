import * as wasm from "elementals";
import { World } from "elementals";

const world = new World();

const CELL_SIZE = 20; // px
const CANVAS_SIZE = 1000; // px

const canvas = document.getElementById("game-canvas");
canvas.height = CANVAS_SIZE;
canvas.width = CANVAS_SIZE;

canvas.addEventListener("mousemove", event => {
    world.set_player_pos(event.offsetX / CELL_SIZE, event.offsetY / CELL_SIZE);
});

const ctx = canvas.getContext("2d");

function draw() {
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    for (let i = 0; i < world.enemies_count(); ++i) {
        const enemy = world.enemy(i);
        ctx.beginPath();
        ctx.arc(enemy.pos.x * CELL_SIZE, enemy.pos.y * CELL_SIZE, CELL_SIZE / 2, 0, 2 * Math.PI);
        ctx.stroke();
        enemy.free();
    }

    const player = world.get_player_pos();

    ctx.beginPath();
    ctx.arc(player.x * CELL_SIZE, player.y * CELL_SIZE, CELL_SIZE / 2, 0, 2 * Math.PI);
    ctx.stroke();

    player.free();
}

setInterval(() => {
    draw();
    world.step();
}, 20);
