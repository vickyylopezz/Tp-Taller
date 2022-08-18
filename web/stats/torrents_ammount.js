fetch("../prueba.json")
.then(response => {
   return response.json();
}).then(jsondata => {

    var dates = []
    jsondata.forEach(element => {
        dates.push(element.upload_date.split(".")[0]);
    });

    dates = dates.sort();

    console.log(dates);

    data_accumulated = [];
    for(var i = 0; i < dates.length; i++){
        data_accumulated.push(i+1);
    }
    console.log(data_accumulated);

    var trace1 = {
        type: "scatter",
        mode: "lines",
        name: 'Torrents Ammount',
        x: dates,
        y: data_accumulated,
        line: {color: '#17BECF'}
      }
      
      var data = [trace1];
      
      var layout = {
        title: 'Torrents ammount by date',
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
      
      

    var elem_cont = document.getElementById("torrents-ammount");
    
    console.log(elem_cont);
    
    const elem = document.createElement("div");
    elem.id = "torrents-ammount-graph";
    elem.style.maxWidth = "550px"
    elem_cont.appendChild(elem);

    Plotly.newPlot('torrents-ammount-graph', data, layout);
});



