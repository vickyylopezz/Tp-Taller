fetch("../prueba.json")
.then(response => {
   return response.json();
}).then(jsondata => {

    var date = {
        date: '',
        ammount_of_peers: 0
    }

    var dates = []
    var dates_started = [];
    var dates_stopped = [];

    jsondata.forEach(element => {
        element.peers.forEach(e => {
            e.interactions.forEach(p => {
                if(p.event == "Stopped"){
                  dates_stopped.push(p.date.split(".")[0])
                }
                else if(p.event == "Started"){
                  dates_started.push(p.date.split(".")[0])
                }
            });
        });
    });

    dates_started = dates_started.sort();
    dates_stopped = dates_stopped.sort();

    var x  = [];
    var y = [];
    var counter = 0;
    var iter = 0;
    for (var i = 0; i < dates_started.length; i++) {
      if(dates_started[i] < dates_stopped[iter]) {
        console.log(dates_started[i])
        x.push(dates_started[i]);
        y.push(counter++);
      } else {
        console.log(dates_stopped[iter]);
        counter--;
        i--;
        iter++;
      }
    }

    console.log(dates_started);
    console.log(dates_stopped);

    var trace1 = {
        type: "scatter",
        mode: "lines",
        name: 'Torrents Ammount',
        x: x,
        y: y,
        line: {color: 'red'}
      }
      
      var data = [trace1];
      
      var layout = {
        title: 'Peers ammount by date',
        xaxis: {
          autorange: true,
          rangeselector: {buttons: [
              {
                count: 1,
                label: '1h',
                step: 'hour',
                stepmode: 'backward'
              },
              {
                count: 5,
                label: '5h',
                step: 'hour',
                stepmode: 'backward'
              },
              {
                count: 1,
                label: '1d',
                step: 'day',
                stepmode: 'backward'
              },
              {
                count: 3,
                label: '3d',
                step: 'day',
                stepmode: 'backward'
              },
              {step: 'all'}
            ]},
          type: 'date'
        },
        yaxis: {
          autorange: true,
          type: 'linear'
        }
      };
      
      

    var elem_cont = document.getElementById("connected-peers");
    
    console.log(elem_cont);
    
    const elem = document.createElement("div");
    elem.id = "connected-peers-graph";
    elem.style.maxWidth = "550px"
    elem_cont.appendChild(elem);

    Plotly.newPlot('connected-peers-graph', data, layout);
});

function getDateByDate(dates, search_date) {
    return dates.find(d => d.date === search_date);
}