fetch("../prueba.json")
.then(response => {
   return response.json();
}).then(jsondata => {
   var table = document.getElementById("bt-table-body");

   for(var i = 0; i < jsondata.length; i++ ){
      var tr = document.createElement('tr');
     
      var th = document.createElement("th");
      th.scope = "row";
      th.innerText = (i+1).toString();

      var hash_info = document.createElement("td");
      hash_info.innerText = jsondata[i].info_hash;

      var dowloading = document.createElement("td");
      var ammount_dowloading = 0;
      var ammount_dowloaded = 0;
      jsondata[i].peers.forEach(element => {
         element.interactions.forEach(e => {
            if (e.event == "Completed"){
               ammount_dowloaded++;
               ammount_dowloading--;
            }
            else if (e.event == "Started"){
               ammount_dowloading++;
            }
         });
      }); 
      dowloading.innerText = ammount_dowloading;

      var downloaded = document.createElement("td");
      downloaded.innerText = ammount_dowloaded;

      tr.appendChild(th);
      tr.appendChild(hash_info);
      tr.appendChild(dowloading);
      tr.appendChild(downloaded);

      table.appendChild(tr);
   }
});

