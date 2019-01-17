import * as wasm from "elementals";
import { World } from "elementals";

const world = new World();
//const point = wasm.greet();
console.log(world.enemies_count());
console.log(world.enemy(0));
console.log(world.enemy(0).pos.x);
//console.log(point.y);
