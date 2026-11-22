/** @jsx CreateElement */
/** @jsxFrag Fragment */
// ============================================================================
// TORTURE TEST V3 (JSX / TSX): 100% COMPILER-VALID (`tsc --strict` ERROR-FREE)
// HORRIFIC K&R / PRETTIER BRAIN-ROT SLOP STRESS TEST FOR LAFS FORMATTERS
// ============================================================================
/** @jsxRuntime classic */
/** @jsx CreateElement */
/** @jsxFrag Fragment */
export type Spectrum_Hex_Token =
| '#7100FF' | '#7100FF22' | '#FC6A03' | '#FF1744'
| '#00BFFF' | '#00E676' | '#C77DFF' | '#3D5AFE' | '#FFFFFF';

export interface Virtual_Node
{
    readonly type: string | Function;
    readonly props: Record<string, unknown>;
    readonly children: readonly unknown[];
}

export const Fragment = Symbol('Fragment');
export function CreateElement(type: string | Function | symbol, props: Record<string, unknown> | null, ...children: unknown[]): Virtual_Node
{
    return { type: typeof type === 'symbol' ? 'Fragment' : type, props: props ?? {}, children };
}

declare global
{
    namespace JSX { interface Element extends Virtual_Node {} interface IntrinsicElements { [elemName: string]: Record<string, unknown>; } }
}

export interface Telemetry_Row_Contract
{
    readonly id: string;
    readonly nodeName: string;
    readonly severity: 'OK' | 'WARN' | 'CRITICAL';
    readonly accentHex: Spectrum_Hex_Token;
    readonly latencyMs: number;
    readonly tags: readonly string[];
}
// 1. TORTURE: `.tsx` Disambiguated Generic Arrow `<const T,>` with K&R Hugging
export const wrapReadonlyTuple = <const T,>(value: T, badgeHex: Spectrum_Hex_Token = '#7100FF'): readonly [T, Spectrum_Hex_Token] => ([value, badgeHex] as const);
// 2. TORTURE: Prematurely Exploded Tiny Leaf Component + Empty Callback Props
export function StatusDotBadge(props: { readonly hex: Spectrum_Hex_Token; readonly label: string; }): JSX.Element
{
    return (
               <span
                   className = "StatusDot"
                   style     = {{ backgroundColor: props.hex }}
               >
                   { props.label }
               </span>
           );
}
// 3. TORTURE: Generic TSX Table Component with Misaligned Attributes & Render-Prop Clumping
export function TelemetryDataGrid<TRow extends { readonly id: string; readonly accentHex: Spectrum_Hex_Token; }>
(
    props:
    {
        readonly rows: readonly TRow[];
        readonly headerTitle: string;
        readonly onRowSelect?: (row: TRow, index: number) => void;
        readonly renderCell: (row: TRow, index: number) => JSX.Element;
    }
): JSX.Element
{
    // Single-Line Complexity Budget Breach (6 statements + 3 nesting depth on 1 line):
    const computeGridChecksum = (count: number): number =>
    {
        let s = 0;
        let m = 1;
        let k = 2;
        let r = 3;
        let p = 4;
        if (count > 0)
        {
            for (let i = 0; i < count; i++) { if (i % 2 === 0) { s += i + m; k ^= s; r += k; p |= r; } }
        }
        return s + p;
    };
    return (
               <section className = "TelemetryGridShell"
                   data-grid-title     = { props.headerTitle }
                   data-checksum       = { computeGridChecksum(props.rows.length) }
                   data-accent-primary = "#7100FF"
                   data-accent-alpha   = "#7100FF22"
                   style               = {{ borderColor: '#7100FF', backgroundColor: '#7100FF22', padding: '16px' }}>
                   <header className = "GridHeader"
                       data-count = { props.rows.length }>
                       <h2>{ props.headerTitle }</h2>
                   </header>
                   <div className = "GridBody">
                       {
                           props.rows.length === 0
                           ?
                           (
                               <div className = "EmptyState" data-tone = "#FC6A03">
                                   No telemetry rows available
                               </div>
                           )
                           :
                           (
                               props.rows.map
                               (
                                   (row, idx) =>
                                   (
                                       <div key = { row.id }
                                           className      = "GridRowItem"
                                           data-row-id    = { row.id }
                                           data-row-index = { idx }
                                           data-row-hex   = { row.accentHex }
                                           onClick        =
                                           {
                                               () => { if (props.onRowSelect) { props.onRowSelect(row, idx); } }
                                           }
                                       >
                                           { props.renderCell(row, idx) }
                                       </div>
                                   )
                               )
                           )
                       }
                   </div>
               </section>
           );
}
// 4. TORTURE: THE ABYSSAL TSX DASHBOARD (Fragments, Inline IIFEs, Nested Ternaries,
// Logical `&&` Chains, Spread Props, Clinging Backticks, & 10-Level Closing Pyramids)
export function AbyssalTelemetryDashboard
(
    props:
    {
        readonly clusterId: string;
        readonly isLiveStreamActive: boolean;
        readonly rows: readonly Telemetry_Row_Contract[];
        readonly extraAriaAttributes: Record<string, string>;
        readonly onEscalateCritical:
        (
            payload:
            {
                readonly clusterId: string;
                readonly criticalIds: readonly string[];
                readonly alertHex: Spectrum_Hex_Token;
            }
        ) => void;
    }
): JSX.Element
{
    // Uninterrupted Sole-Child Chain (Should fuse into `[({` and `)}]` in LAFS State 2!):
    const staticThemeMatrix =
    [({
        tokens:
        [({
            primary: '#7100FF' as const,
            warning: '#FC6A03' as const,
            critical: '#FF1744' as const,
            stream: '#00BFFF' as const,
            healthy: '#00E676' as const,
            secondary: '#C77DFF' as const,
            anchor: '#3D5AFE' as const
        })]
    })];
    // Clinging Backtick Inline CSS Variable Block:
    const inlineCssVariables =
    `
--lafs-primary: #7100FF;
--lafs-primary-alpha: #7100FF22;
--lafs-warning: #FC6A03;
--lafs-critical: #FF1744;
--lafs-cluster-id: "${
    props.clusterId.trim().length > 0
    ? props.clusterId.trim() : 'UNKNOWN_CLUSTER'
}";`;
    return (
               <>
                   <main className = "DashboardViewport"
                       id                      = { `Cluster_${props.clusterId}` }
                       data-live-stream        = { props.isLiveStreamActive ? 'ACTIVE' : 'PAUSED' }
                       data-theme-matrix-count = { staticThemeMatrix.length }
                       data-css-vars           = { inlineCssVariables } { ...props.extraAriaAttributes }>
                       <nav className = "TopControlRibbon"
                           data-role   = "Navigation"
                           data-accent = "#3D5AFE">
                           <button type = "button"
                               className      = "EscalateButton"
                               disabled       = { !props.isLiveStreamActive }
                               data-alert-hex = "#FF1744"
                               onClick        =
                               {
                                   () =>
                                   {
                                       const criticalIds = props.rows
                                       .filter( (r) => r.severity === 'CRITICAL')
                                       .map( (r) => r.id);
                                       if (criticalIds.length > 0)
                                       {
                                           props.onEscalateCritical({ clusterId: props.clusterId, criticalIds, alertHex: '#FF1744' });
                                       }
                                   }
                               }
                           >
                               Escalate Critical Nodes
                           </button>
                           <button type = "button" onClick = { () => {} }>
                               Noop Trigger
                           </button>
                       </nav>{ /* Deeply Nested Logical && + Inline IIFE + Generic Component + Nested Ternaries */ }
                       {
                           props.isLiveStreamActive &&
                           (
                               <div className = "LiveTelemetryStage"
                                   data-stage-hex = "#00BFFF">
                                   {
                                       (
                                           () =>
                                           {
                                               const groupedCount = props.rows.reduce
                                               (
                                                   (acc, item) =>
                                                   ({
                                                       ok: acc.ok + (item.severity === 'OK' ? 1 : 0),
                                                       warn: acc.warn + (item.severity === 'WARN' ? 1 : 0),
                                                       crit: acc.crit + (item.severity === 'CRITICAL' ? 1 : 0)
                                                   }),

                                                   { ok: 0, warn: 0, crit: 0 }
                                               );
                                               return (
                                                          <aside className = "TelemetrySummaryStrip"
                                                              data-ok-count   = { groupedCount.ok }
                                                              data-warn-count = { groupedCount.warn }
                                                              data-crit-count = { groupedCount.crit }>
                                                              <StatusDotBadge hex = "#00E676" label = { `OK: ${groupedCount.ok}` } />
                                                              <StatusDotBadge hex = "#FC6A03" label = { `WARN: ${groupedCount.warn}` } />
                                                              <StatusDotBadge hex = "#FF1744" label = { `CRIT: ${groupedCount.crit}` } />
                                                          </aside>
                                                      );
                                           }
                                       )()
                                   }
                                   <TelemetryDataGrid<Telemetry_Row_Contract>
                                       rows        = { props.rows }
                                       headerTitle = { `Cluster Telemetry [${props.clusterId}]` }
                                       onRowSelect =
                                       {
                                           (selectedRow, selectedIndex) =>
                                           {
                                               if (selectedRow.severity === 'CRITICAL')
                                               {
                                                   props.onEscalateCritical
                                                   ({
                                                       clusterId: props.clusterId,
                                                       criticalIds: [selectedRow.id],
                                                       alertHex: selectedIndex % 2 === 0 ? '#FF1744' : '#FC6A03'
                                                   });
                                               }
                                           }
                                       }
                                       renderCell  =
                                       {
                                           (row, rowIdx) =>
                                           (
                                               <article className = "RowCardContent"
                                                   data-node-name  = { row.nodeName }
                                                   data-latency-ms = { row.latencyMs }
                                                   style           =
                                                   {{
                                                       borderLeftColor: row.severity === 'CRITICAL'
                                                       ? '#FF1744'
                                                       : row.severity === 'WARN'
                                                       ? '#FC6A03'
                                                       : '#00E676',
                                                       color: '#FFFFFF'
                                                   }}
                                               >
                                                   <div className = "RowMetaHeader">
                                                       <strong>{ row.nodeName }</strong>
                                                       <StatusDotBadge
                                                           hex = { row.accentHex }
                                                           label = { row.severity } />
                                                   </div>
                                                   {
                                                       row.tags.length > 0 &&
                                                       (
                                                           <ul className = "TagCloudList">
                                                               {
                                                                   row.tags.map
                                                                   (
                                                                       (tag, tagIdx) =>
                                                                       (
                                                                           <li key = { `${row.id}_${tagIdx}` }
                                                                               data-tag-idx     = { tagIdx }
                                                                               data-parent-idx  = { rowIdx }
                                                                               data-badge-color = "#C77DFF">
                                                                               { wrapReadonlyTuple(tag, '#C77DFF') [0] }
                                                                           </li>
                                                                       )
                                                                   )
                                                               }
                                                           </ul>
                                                       )
                                                   }
                                               </article>
                                           )
                                       }
                                   />
                               </div>
                           )
                       }
                   </main>
               </>
           );
}
