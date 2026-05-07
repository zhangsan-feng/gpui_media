


/*


https://suonizy.net/
https://bfzy5.tv/
https://www.ryzyw.com/
https://lzizy.net/
https://ffzy5.tv/
http://caiji.dyttzyapi.com/
https://jszyapi.com/
https://hongniuziyuan.net/
https://1080zyk6.com
https://yayazy2.com/
https://360zy5.com
https://niuniuzy.cc
https://okzyw.vip
https://qihuzy4.com
https://mtzy.me
https://jszy333.com/
https://dbzy.tv
https://okzyw.cc/
http://www.wujinzy.net/
https://guangsuzy.com/
https://ukuzy0.com/
https://www.xinlangzy.com/
http://kuaichezy.com/
http://jinyingzy.com/
https://www.taopianzy.com/
http://wolongzyw.com/
http://www.ckzy1.com/
https://xinlangzy.com
https://hongniuzy.net
https://haohuazy.com
https://www.taopianzy.com/index.html












https://youzisp.tv
https://www.bttwo.org/
https://www.appmovie.link/
https://dmbus.cc/
https://www.keke2.app/
https://www.xuandm.com/
https://www.renren.pro/

*/
use crate::entity::StreamMedioConvertLayer;

mod youzisp;



pub async fn recommend() -> Vec<StreamMedioConvertLayer>{
    let mut  call_back = Vec::new();
    call_back.extend(youzisp::recommend::recommend().await);
    
    call_back

}