fetch("../prueba.json")
.then(response => {
   return response.json();
}).then(jsondata => {

    var date = {
        date: '',
        ammount_of_peers: 0
    }

    var dates = []

    jsondata.forEach(element => {
        element.peers.forEach(e => {
            e.interactions.forEach(p => {
                var d = getDateByDate(dates,p.date);              
                if(p.event == "Completed"){
                    if(!d){
                        dates.push({
                          
                            date: p.date.split(".")[0],
                            ammount_of_peers: 1
                        })
                    }
                    else{
                        d.ammount_of_peers++;
                    }
                }
            });
        });
    });

    console.log(dates);

    var x  = [];
    var y = [];
    
    for(var i = 0; i < dates.length; i++){
        x.push(dates[i].date);
        if(i ==0){
            y.push(dates[i].ammount_of_peers);
        }else{
            y.push(dates[i].ammount_of_peers+y[i-1]);
        }
    }
    x = x.sort();
    var trace1 = {
        x: x,
        y: y,
        name: 'Peers completed',
        type: 'bar',
        marker: {
          color: 'rgb(158,202,225)',
          opacity: 0.6,
          line: {
            color: 'rgb(8,48,107)',
            width: 1.5
          }
        }
    };
    console.log("COMPLETE PEERS DATES: " + trace1.x);

      var data = [trace1];
      
      var layout = {
        title: 'Peers completed ammount by date',
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
      

    var elem_cont = document.getElementById("completed-peers");
    
    console.log(elem_cont);
    
    const elem = document.createElement("div");
    elem.id = "completed-peers-graph";
    elem.style.maxWidth = "550px"
    elem_cont.appendChild(elem);

    Plotly.newPlot('completed-peers-graph', data, layout);
});

function getDateByDate(dates, search_date) {
    return dates.find(d => d.date === search_date);
}

