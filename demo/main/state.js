import { cloneSpec } from "./shared.js";
import {
  initialBarSpec,
  initialHeatmapSpec,
  initialLineSpec,
  initialNetworkSpec,
  initialScatterSpec,
  initialTreeSpec
} from "./specs.js";

export const state = {
  scatterSpec: cloneSpec(initialScatterSpec),
  treeSpec: cloneSpec(initialTreeSpec),
  lineSpec: cloneSpec(initialLineSpec),
  barSpec: cloneSpec(initialBarSpec),
  heatmapSpec: cloneSpec(initialHeatmapSpec),
  networkSpec: cloneSpec(initialNetworkSpec),
  scatterSessionHandle: null,
  scatterRenderFrame: null,
  treeSessionHandle: null,
  treeTransitionFrame: null,
  lineSessionHandle: null,
  barSessionHandle: null,
  heatmapSessionHandle: null,
  networkSessionHandle: null,
  scatterPerf: {
    pointCount: initialScatterSpec.points.length,
    renderMs: 0,
    mode: "sample"
  }
};
