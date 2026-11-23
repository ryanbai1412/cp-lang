import sys,json,subprocess,os,glob,random,importlib
sys.path.insert(0,os.path.dirname(os.path.abspath(__file__)))
C=os.environ.get('CPL','cpl')
D=os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
B=os.environ.get('OUT','/tmp/r31-bin');os.makedirs(B,exist_ok=True)
def interp(f,inp): 
    r=subprocess.run([C,'run',f],input=inp,capture_output=True,text=True,timeout=120);return r.stdout,r.stderr,r.returncode
def native(f,inp,exe):
    r=subprocess.run([exe],input=inp,capture_output=True,text=True,timeout=60);return r.stdout,r.stderr,r.returncode
def build(f,flags,tag):
    cpp=os.path.join(B,os.path.basename(f)[:-3]+'.cpp');exe=os.path.join(B,os.path.basename(f)[:-3]+tag)
    r=subprocess.run([C,'build',f,'-o',cpp],capture_output=True,text=True)
    if r.returncode: return None,'cpl build: '+r.stderr[:800]
    r=subprocess.run(['g++']+flags+[cpp,'-o',exe],capture_output=True,text=True)
    if r.returncode: return None,'g++: '+r.stderr[:1500]
    return exe,''
