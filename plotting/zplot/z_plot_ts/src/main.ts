import { Config, Session, Sample } from "@ZettaScaleLabs/zenoh-ts";
import * as Plotly from 'plotly.js-dist-min';

const session : Session = await Session.open(Config.new("ws/127.0.0.1:10000"));

Plotly.newPlot('plot', [{x: [], y: [], mode: 'lines'}]);

await session.declare_subscriber(
  "demo/random",
  async function (sample: Sample): Promise<void> {
    var plot = document.getElementById('plot') as Plotly.PlotlyHTMLElement;
    if (plot) {
      var traceIdx = plot.data.findIndex((v: Plotly.Data) => v.name === sample.keyexpr().toString());
      if (traceIdx == -1 ) {
          Plotly.addTraces('plot', {x: [], y: [], name: sample.keyexpr().toString(), mode: 'lines'}, 0);
          traceIdx = 0;
      }

      var time = new Date();
      var timestamp = sample.timestamp()
      if (timestamp) {
        time = new Date(parseInt(timestamp.split('/')[0]) / 4294967.295)
      }

      var update = {
          x: [[time]],
          y: [[new TextDecoder().decode(sample.payload().buffer())]],
      }

      var olderTime = time.setMinutes(time.getMinutes() - 1);
      var futureTime = time.setMinutes(time.getMinutes() + 1);

      var minuteView: Partial<Plotly.Layout> = {
          xaxis: {
              type: 'date',
              range: [olderTime,futureTime]
          }
      };

      Plotly.relayout('plot', minuteView);
      Plotly.extendTraces('plot', update, [traceIdx]);
    }
  }
);
